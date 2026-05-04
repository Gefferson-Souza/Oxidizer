# 9. Mutex Re-entrance Protocol for Generated `@Injectable` State

Date: 2026-05-03
Status: Accepted (retroactive — backfill of PR #141)

## Context

`std::sync::Mutex` is **non-reentrant**: a second `.lock()` call from the same thread while a guard is alive deadlocks. Tyrus emits NestJS `@Injectable` services where private fields become `Arc<Mutex<T>>`-wrapped state. Two patterns in pre-PR-#141 codegen triggered the deadlock:

1. **Compound assigns on state fields.** `this.counter += 1` (and `-=`/`*=`/`/=`/`%=`/bitwise/shift) silently mis-compiled because the `StateField` write branch ignored the binary operator and emitted a plain `=`. Even when the codegen *did* route through a read-then-write split, the read guard and the write guard were both alive across the assign expression — second `.lock()` deadlocked.

2. **Dual reads in one statement.** `User { id: this.x, count: this.x }` and `Math.max(this.x, this.x)` kept two `MutexGuard` temporaries alive until the enclosing statement's terminating `;` — Rust's drop semantics would only release them at end-of-statement, not at the consuming expression. The second `.lock()` blocked indefinitely.

Both bugs surfaced under realistic NestJS workloads (counters, rate limiters, dashboards). Neither was caught by snapshot tests because the generated Rust *compiled*; only at runtime, on the second invocation of the controller method, did the worker thread hang.

PR #141 (`b5af050`) closed both bugs by establishing a uniform emission protocol. This ADR documents that protocol so future contributors — particularly anyone implementing #143 (atomic state fields) or revisiting `convert/expr/member.rs` / `convert/expr/misc.rs` — know the invariants are load-bearing, not incidental.

## Decision

Generated Rust for `@Injectable` state fields obeys two emission patterns. Both are implemented in `crates/tyrus_codegen/src/convert/expr/`:

### Pattern A — Block-scoped read

Every read of `this.field` where `field` is a state-field expands to:

```rust
{ let __g = self.field.lock().unwrap(); *__g }
```

(or `.clone()` for non-`Copy` payloads). The guard `__g` is bound in an **inner block**, so Rust's drop scope ends at the closing `}` of that block — *before* the consuming expression evaluates the next subexpression. Two reads in the same statement therefore release each guard between reads.

This is implemented in `convert_this_member` and propagated through `convert_member_expr` for every `Expr::Member` whose object resolves to `self` and whose property is registered in `current_class_state_fields`.

### Pattern B — Read-then-write split for compound writes

Compound assignments and update expressions (`+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`, prefix/postfix `++`/`--`) on state fields expand into two statements:

```rust
let __cur = { let __g = self.field.lock().unwrap(); *__g };
let __new = __cur <op> rhs;
{ let mut __g = self.field.lock().unwrap(); *__g = __new; }
```

The read guard drops at the `;` of its own `let`. The write guard drops at the `;` of the assign block. The operator is preserved (no silent reduction to `=`). Update expressions (`this.x++`) route through the same helper (`emit_state_field_assign`) for uniform semantics.

### Invariant

For any generated function body emitted from a `&mut self` method on an `@Injectable` class, **at most one `MutexGuard` for any given state field is alive at any program point**. The compiler does not enforce this — codegen does. Any new emission path that touches state fields **must** route through the existing helpers (`emit_state_field_read`, `emit_state_field_assign`) or replicate the patterns above explicitly.

## Consequences

### Positive

- **Deadlock-free under realistic workloads.** The two failure patterns that triggered #141 are architecturally impossible under the protocol.
- **Predictable mental model.** "Each read = one guard, dropped at end of inner block. Each write = read-block then write-block." Reviewers do not need to reason about `MutexGuard` drop order across complex expressions.
- **Forward compatibility.** When #143 lands atomic state fields (`AtomicU64` etc.), the `emit_state_field_*` helpers are the single substitution point — no code outside them needs revision.

### Negative

- **Slight overhead.** Every state-field read costs a block scope and a deref vs the naive `self.field.lock().unwrap().clone()`. In practice negligible (the same guard acquisition is the dominant cost). Atomic state (#143) eliminates the overhead for `Copy` payloads.
- **Snapshot churn.** PR #141 updated the tier4 NestJS controller and injectable-service snapshots to the guarded-block form. Future codegen tweaks affecting state reads will keep producing snapshot diffs.
- **Codegen complexity.** The path from `Expr::Member { obj: This, prop: Ident }` to emitted tokens passes through three helpers and one tracked HashSet (`current_class_state_fields`). Anyone adding a new mutating expression (e.g., a hypothetical `Math.atomic_add`) must explicitly handle the state-field branch.

## Alternatives rejected

- **`parking_lot::ReentrantMutex`.** Would lift the constraint at the cost of (a) extra dependency in every generated `Cargo.toml`, (b) loss of `Send + Sync` guarantees that NestJS `@Injectable` services rely on for axum's `with_state`, (c) silent acceptance of recursive locking — exactly the pattern Rule 1 (bounded control flow) discourages. Rejected.
- **Skip-on-error pattern (`if let Ok(g) = self.field.try_lock())`.** Trades deadlock for silent data corruption (lost writes). Worse outcome. Rejected.
- **Compile-time check via macro (`#[state]`).** Would require a procedural macro on every generated field. Adds a build dependency and a parsing step we control end-to-end via codegen. The codegen-side discipline is the simpler enforcement. Rejected for now; revisitable if user-authored Rust ever needs the same protection.

## References

- Originating PR: [#141](https://github.com/Gefferson-Souza/Tyrus/pull/141) (`b5af050`) — `fix(codegen): prevent mutex re-entrance on compound assigns and dual reads`. Closes #129.
- Implementation: `crates/tyrus_codegen/src/convert/expr/member.rs`, `crates/tyrus_codegen/src/convert/expr/misc.rs`.
- Regression tests: `tests/src/unit/state_mutation.rs` (8 tests).
- Related: ADR 0008 (Tyrus Strict Rules — this ADR satisfies Rule 10's retroactive requirement); future ADR for #143 (atomic state) supersedes Pattern B for `Copy` payloads.
