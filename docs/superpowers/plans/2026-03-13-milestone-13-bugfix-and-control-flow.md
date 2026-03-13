# Milestone 13: Bug Fixes + Control Flow Expansion

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 4 known transpiler bugs and unlock 6 blocked control flow constructs, expanding the Oxidizable Standard coverage from ~70% to ~90%.

**Architecture:** Changes target 3 layers: (1) `tyrus_analyzer` to unblock rejected statements, (2) `tyrus_codegen` to fix bugs and add missing transpilation, (3) `tests/` for comprehensive validation. All changes follow TDD — write failing test first, then fix.

**Tech Stack:** Rust, SWC AST (`swc_ecma_ast`, `swc_ecma_visit`), `quote!` macros, `insta` snapshots, `cargo nextest`.

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `crates/tyrus_analyzer/src/lints.rs` | Modify | Remove rejections for for-of, for-in, do-while, for, switch, try-catch |
| `crates/tyrus_codegen/src/convert/stmt.rs` | Modify | Add switch, try-catch, do-while, traditional for transpilation |
| `crates/tyrus_codegen/src/convert/expr/mod.rs` | Modify | Add unary expression dispatch |
| `crates/tyrus_codegen/src/convert/expr/misc.rs` | Modify | Fix optional chaining `Some()` wrapper, add unary expr conversion |
| `crates/tyrus_codegen/src/convert/expr/call.rs` | Modify | Fix `.find()` closure type mismatch |
| `crates/tyrus_codegen/src/convert/expr/arrow.rs` | Modify | Fix `const` vs `let` immutability (if variable decl logic is here) |
| `crates/tyrus_codegen/src/convert/interface.rs` | Modify | Fix `const` → `let` (immutable) in variable declarations |
| `tests/fixtures/tier5/` | Create | New fixtures for control flow + bug fixes |
| `tests/src/unit/tier5.rs` | Create | Unit tests for new features |
| `tests/src/snapshot/tier5.rs` | Create | Snapshot tests for new features |
| `tests/src/compilation/tier5.rs` | Create | Compilation tests for new features |

---

## Chunk 1: Bug Fixes (4 bugs)

### Task 1: Fix `const` vs `let` immutability

**Context:** Currently both `const x = 1` and `let x = 1` produce `let mut x = 1f64`. The fix: `const` → `let` (immutable), `let` → `let mut` (mutable).

**Files:**
- Test: `tests/src/unit/tier5.rs`
- Modify: `crates/tyrus_codegen/src/convert/stmt.rs` (variable declaration logic)
- Modify: `crates/tyrus_codegen/src/convert/interface.rs` (if VarDecl is handled in Visit impl)

**Investigation needed:** Before implementing, read `stmt.rs` and `interface.rs` to find where `VarDecl` is processed and where `let mut` is emitted. The fix is changing the emission logic based on `decl.kind` (`VarDeclKind::Const` vs `VarDeclKind::Let`).

- [ ] **Step 1: Write failing test**

In `tests/src/unit/tier5.rs`:
```rust
#[test]
fn test_const_produces_immutable_let() {
    let rust = transpile("function f(): void { const x: number = 42; }");
    assert!(rust.contains("let x"), "const should produce immutable 'let'");
    assert!(!rust.contains("let mut x"), "const should NOT produce 'let mut'");
}

#[test]
fn test_let_produces_mutable_let_mut() {
    let rust = transpile("function f(): void { let x: number = 42; }");
    assert!(rust.contains("let mut x"), "let should produce 'let mut'");
}
```

- [ ] **Step 2: Run tests — expect FAIL** (const currently produces `let mut`)

Run: `cargo test -p integration_tests tier5::test_const_produces_immutable_let -- --exact`

- [ ] **Step 3: Find and fix the VarDecl emission logic**

Read `stmt.rs` to find where `VarDeclKind` is checked. Change:
- `VarDeclKind::Const` → emit `let` (no `mut`)
- `VarDeclKind::Let` → emit `let mut`

- [ ] **Step 4: Run tests — expect PASS**

- [ ] **Step 5: Update snapshot tests** (existing snapshots will change since `const` now produces `let`)

Run: `cargo insta review`

---

### Task 2: Fix unary expression support (`!`, `-`, `+`)

**Context:** Unary expressions (`-x`, `!flag`, `+str`) hit `compile_error!("Tyrus: unsupported expression")` in the expression dispatcher.

**Files:**
- Test: `tests/src/unit/tier5.rs`
- Modify: `crates/tyrus_codegen/src/convert/expr/mod.rs` (add `Expr::Unary` dispatch)
- Modify: `crates/tyrus_codegen/src/convert/expr/misc.rs` (add `convert_unary_expr` function)

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_unary_negation() {
    let rust = transpile("function f(x: number): number { return -x; }");
    assert!(rust.contains("-x") || rust.contains("- x"), "should preserve unary negation");
    assert!(!rust.contains("compile_error"), "should NOT produce compile_error");
}

#[test]
fn test_unary_not() {
    let rust = transpile("function f(x: boolean): boolean { return !x; }");
    assert!(rust.contains("!x") || rust.contains("! x"), "should preserve logical NOT");
}

#[test]
fn test_unary_plus() {
    let rust = transpile("function f(x: number): number { return +x; }");
    // Unary + in JS is identity for numbers, can be dropped
    assert!(!rust.contains("compile_error"), "should NOT produce compile_error");
}
```

- [ ] **Step 2: Run tests — expect FAIL**

- [ ] **Step 3: Add `Expr::Unary` to expression dispatcher**

In `expr/mod.rs`, add a match arm:
```rust
Expr::Unary(unary) => convert_unary_expr(unary, generator),
```

- [ ] **Step 4: Implement `convert_unary_expr` in `expr/misc.rs`**

```rust
pub(crate) fn convert_unary_expr(
    expr: &UnaryExpr,
    generator: &RustGenerator,
) -> proc_macro2::TokenStream {
    let arg = convert_expr(&expr.arg, generator);
    match expr.op {
        UnaryOp::Minus => quote::quote! { -#arg },
        UnaryOp::Plus => arg, // identity for numbers
        UnaryOp::Bang => quote::quote! { !#arg },
        UnaryOp::TypeOf => quote::quote! { compile_error!("Tyrus: typeof not supported") },
        _ => quote::quote! { compile_error!("Tyrus: unsupported unary operator") },
    }
}
```

- [ ] **Step 5: Run tests — expect PASS**

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "fix(codegen): add unary expression support (!, -, +)"
```

---

### Task 3: Fix optional chaining `Some()` wrapper

**Context:** Optional chaining `obj?.name` generates `and_then(|__v| __v.name.clone())` but should generate `and_then(|__v| Some(__v.name.clone()))` for non-Option fields.

**Files:**
- Test: `tests/src/compilation/tier5.rs` (the ignored test should now pass)
- Modify: `crates/tyrus_codegen/src/convert/expr/misc.rs` (find `convert_opt_chain`)

- [ ] **Step 1: Read `misc.rs` to find the optional chaining logic**

- [ ] **Step 2: Un-ignore the existing test**

In `tests/src/compilation/tier3.rs`, remove `#[ignore]` from `test_tier3_optional_chaining_compiles`.

- [ ] **Step 3: Fix the closure to wrap return in `Some()`**

In the `and_then` closure generation, change the body from:
```rust
quote! { |__v| #body }
```
to:
```rust
quote! { |__v| Some(#body) }
```

- [ ] **Step 4: Run tests — expect PASS**

- [ ] **Step 5: Commit**

---

### Task 4: Fix `.find()` closure type mismatch

**Context:** `.find()` generates `nums.iter().find(|n| n > 10f64)` but `n` is `&&f64` (double reference), causing type mismatch.

**Files:**
- Test: `tests/src/compilation/tier3.rs` (un-ignore the test)
- Modify: `crates/tyrus_codegen/src/convert/expr/call.rs` (find `.find()` generation)

- [ ] **Step 1: Read `call.rs` to find the `.find()` logic**

- [ ] **Step 2: Fix the closure parameter dereferencing**

The fix is to dereference in the closure: `|n| *n > 10f64` or `|&&n| n > 10f64`

Preferred approach — add dereference in comparison:
```rust
// For .find(), generate |item| *item > value
quote! { |#param| *#param #op #value }
```

Or use `.iter().find(|&#param| ...)` pattern.

- [ ] **Step 3: Un-ignore `test_tier3_advanced_methods_compiles`**

- [ ] **Step 4: Run tests — expect PASS**

- [ ] **Step 5: Commit**

---

## Chunk 2: Unlock Blocked Control Flow

### Task 5: Remove analyzer rejections for supported control flow

**Context:** `lints.rs` explicitly rejects 6 control flow constructs that already have (or will have) codegen support. We need to selectively unblock them.

**Files:**
- Modify: `crates/tyrus_analyzer/src/lints.rs`
- Test: `tests/src/unit/tier5.rs`

- [ ] **Step 1: Read `lints.rs` to find all rejection points**

Look for `visit_for_stmt`, `visit_for_of_stmt`, `visit_for_in_stmt`, `visit_do_while_stmt`, `visit_switch_stmt`, `visit_try_stmt`.

- [ ] **Step 2: Remove/comment the rejection visitors for:**
- `for_of_stmt` (codegen already exists in `stmt.rs`)
- `for_in_stmt` (codegen already exists in `stmt.rs`)
- `do_while_stmt` (will add codegen in Task 7)
- `for_stmt` (will add codegen in Task 6)
- Keep rejection for `switch` and `try-catch` until codegen is ready (Tasks 8-9)

- [ ] **Step 3: Write test for for-of loop**

```rust
#[test]
fn test_for_of_loop() {
    let rust = transpile("function f(): void { const arr: number[] = [1,2,3]; for (const x of arr) { console.log(x); } }");
    assert!(rust.contains("for"), "should contain for loop");
    assert!(!rust.contains("compile_error"), "should NOT reject for-of");
}
```

- [ ] **Step 4: Run tests — expect PASS** (codegen already exists)

- [ ] **Step 5: Commit**

```bash
git commit -m "feat(analyzer): unblock for-of, for-in, do-while, for loops"
```

---

### Task 6: Add traditional `for` loop transpilation

**Context:** `for (let i = 0; i < n; i++)` → Rust `for` or `while` loop equivalent.

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/stmt.rs`
- Test: `tests/src/unit/tier5.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_traditional_for_loop() {
    let rust = transpile("function f(n: number): number { let sum: number = 0; for (let i: number = 0; i < n; i++) { sum = sum + i; } return sum; }");
    assert!(!rust.contains("compile_error"), "should transpile for loop");
    // Traditional for becomes while in Rust
    assert!(rust.contains("while") || rust.contains("for"), "should produce loop");
}
```

- [ ] **Step 2: Implement `ForStmt` handling in `stmt.rs`**

Strategy: Convert `for (init; test; update) { body }` to:
```rust
{
    init;
    while test {
        body;
        update;
    }
}
```

- [ ] **Step 3: Run tests — expect PASS**

- [ ] **Step 4: Commit**

---

### Task 7: Add `do-while` loop transpilation

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/stmt.rs`
- Test: `tests/src/unit/tier5.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_do_while_loop() {
    let rust = transpile("function f(): number { let x: number = 0; do { x = x + 1; } while (x < 10); return x; }");
    assert!(rust.contains("loop"), "do-while should use Rust 'loop'");
    assert!(rust.contains("break"), "do-while should break on condition");
}
```

- [ ] **Step 2: Implement `DoWhileStmt` in `stmt.rs`**

Strategy: Convert `do { body } while (test)` to:
```rust
loop {
    body;
    if !test { break; }
}
```

- [ ] **Step 3: Run tests — expect PASS**

- [ ] **Step 4: Commit**

---

### Task 8: Add `switch` statement transpilation

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/stmt.rs`
- Modify: `crates/tyrus_analyzer/src/lints.rs` (remove rejection)
- Test: `tests/src/unit/tier5.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_switch_statement() {
    let rust = transpile(r#"function f(x: string): string {
        switch (x) {
            case "a": return "alpha";
            case "b": return "beta";
            default: return "unknown";
        }
    }"#);
    assert!(rust.contains("match"), "switch should become match");
    assert!(rust.contains("alpha"), "should contain case body");
}
```

- [ ] **Step 2: Remove switch rejection from `lints.rs`**

- [ ] **Step 3: Implement `SwitchStmt` in `stmt.rs`**

Strategy: Convert `switch(x) { case "a": ...; default: ... }` to:
```rust
match x.as_str() {  // or match x { } for non-string types
    "a" => { ... },
    "b" => { ... },
    _ => { ... },
}
```

Handle fall-through by collecting consecutive cases without break into the same arm.

- [ ] **Step 4: Run tests — expect PASS**

- [ ] **Step 5: Commit**

---

### Task 9: Add `try-catch` transpilation

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/stmt.rs`
- Modify: `crates/tyrus_analyzer/src/lints.rs` (remove rejection)
- Test: `tests/src/unit/tier5.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn test_try_catch() {
    let rust = transpile(r#"function f(): string {
        try {
            return "ok";
        } catch (e) {
            return "error";
        }
    }"#);
    assert!(rust.contains("match") || rust.contains("if let"), "try-catch should use match/if-let");
    assert!(!rust.contains("compile_error"), "should NOT produce compile_error");
}
```

- [ ] **Step 2: Remove try-catch rejection from `lints.rs`**

- [ ] **Step 3: Implement `TryStmt` in `stmt.rs`**

Strategy: Convert `try { body } catch(e) { handler }` to:
```rust
match (|| -> Result<_, Box<dyn std::error::Error>> { body })() {
    Ok(val) => val,
    Err(e) => { handler },
}
```

Or simpler: wrap the try body in a closure that returns Result, then match.

- [ ] **Step 4: Run tests — expect PASS**

- [ ] **Step 5: Commit**

---

## Chunk 3: Stdlib Expansion

### Task 10: Add missing Array methods

**Files:**
- Modify: `crates/tyrus_codegen/src/stdlib/array.rs`
- Test: `tests/src/unit/tier5.rs`

Methods to add:
- `concat()` → `.iter().chain(other.iter()).cloned().collect()`
- `slice(start, end)` → `[start..end].to_vec()`
- `includes(item)` → `.contains(&item)`
- `indexOf(item)` → `.iter().position(|x| x == &item)`
- `length` (property) → `.len()`

- [ ] **Step 1: Write failing tests for each method**
- [ ] **Step 2: Implement in `array.rs`**
- [ ] **Step 3: Run tests — expect PASS**
- [ ] **Step 4: Commit**

---

### Task 11: Add missing String methods

**Files:**
- Modify: `crates/tyrus_codegen/src/stdlib/string.rs`
- Test: `tests/src/unit/tier5.rs`

Methods to add:
- `substring(start, end)` → `[start..end]` or `.get(start..end)`
- `charAt(i)` → `.chars().nth(i)`
- `startsWith(prefix)` → `.starts_with(&prefix)`
- `endsWith(suffix)` → `.ends_with(&suffix)`
- `repeat(n)` → `.repeat(n)`
- `padStart(len, fill)` → `format!("{:>width$}", s, width=len)` with fill
- `length` (property) → `.len()`

- [ ] **Step 1: Write failing tests for each method**
- [ ] **Step 2: Implement in `string.rs`**
- [ ] **Step 3: Run tests — expect PASS**
- [ ] **Step 4: Commit**

---

### Task 12: Add Object methods

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/expr/call.rs` (or new `stdlib/object.rs`)
- Test: `tests/src/unit/tier5.rs`

Methods to add:
- `Object.keys(obj)` → `obj.keys().cloned().collect::<Vec<_>>()` (for HashMap)
- `Object.values(obj)` → `obj.values().cloned().collect::<Vec<_>>()`
- `Object.entries(obj)` → `obj.iter().collect::<Vec<_>>()`

- [ ] **Step 1: Write failing tests**
- [ ] **Step 2: Implement**
- [ ] **Step 3: Run tests — expect PASS**
- [ ] **Step 4: Commit**

---

## Chunk 4: Integration Tests + Snapshot Updates

### Task 13: Create tier5 test fixtures

**Files:**
- Create: `tests/fixtures/tier5/control_flow.ts` (for, for-of, do-while, switch)
- Create: `tests/fixtures/tier5/error_handling.ts` (try-catch)
- Create: `tests/fixtures/tier5/unary_ops.ts` (!, -, +)
- Create: `tests/fixtures/tier5/immutability.ts` (const vs let)

- [ ] **Step 1: Write fixture files**
- [ ] **Step 2: Add snapshot tests in `tests/src/snapshot/tier5.rs`**
- [ ] **Step 3: Add compilation tests in `tests/src/compilation/tier5.rs`**
- [ ] **Step 4: Register modules in `mod.rs` files**
- [ ] **Step 5: Run all tests — expect PASS**
- [ ] **Step 6: Accept snapshots with `cargo insta accept`**
- [ ] **Step 7: Final commit**

```bash
git commit -m "test(tier5): add fixtures and tests for control flow + bug fixes"
```

---

## Chunk 5: Documentation + Cleanup

### Task 14: Update all documentation

- [ ] **Step 1: Update ROADMAP.md** — Add Milestone 13
- [ ] **Step 2: Update GRAMMAR.md** — Add switch, try-catch, do-while, for to supported constructs
- [ ] **Step 3: Update CLAUDE.md** — Update test count and known limitations
- [ ] **Step 4: Update STANDARDIZATION_PLAN.md** — Mark Phase 8 (if applicable)
- [ ] **Step 5: Update benches/README.md** — If any new benchmark scenarios

### Task 15: Final verification

- [ ] **Step 1: Run full test suite** — `cargo nextest run --workspace`
- [ ] **Step 2: Run clippy** — `cargo clippy --workspace`
- [ ] **Step 3: Run fmt check** — `cargo fmt -- --check`
- [ ] **Step 4: Verify demo** — `cargo run --bin tyrus -- build examples/real_world_demo/src --output examples/real_world_demo/output && cd examples/real_world_demo/output && cargo check`

---

## Summary

| Chunk | Tasks | Impact |
|-------|-------|--------|
| 1: Bug Fixes | 4 bugs fixed | `const` immutability, unary ops, optional chaining, find() |
| 2: Control Flow | 5 constructs unlocked | for, for-of, for-in, do-while, switch, try-catch |
| 3: Stdlib | 15+ methods added | Array, String, Object methods |
| 4: Tests | 20+ new tests | Tier 5 fixtures + compilation verification |
| 5: Docs | Update all | ROADMAP, GRAMMAR, CLAUDE.md |

**Expected outcome:** TypeScript coverage jumps from ~70% to ~90% of the Oxidizable Standard.
