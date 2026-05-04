# 10. Formatter Contract for `tyrus_orchestrator::format`

Date: 2026-05-03
Status: Accepted (retroactive — backfill of PR #142)

## Context

Rule 7 (POWER_OF_TEN.md) forbids string concatenation as a Rust-emission mechanism anywhere under `crates/tyrus_codegen/src/convert/`. The single allowed exception is `crates/tyrus_orchestrator/src/format.rs` — it owns formatting/layout of pre-validated source and the hard-coded `&'static str` blocks for `AppError` (`get_app_error_simple`, `get_app_error_code`).

Because `format::format_code` is the **last gate** before generated `.rs` files hit disk, three properties are load-bearing:

1. **Idempotence.** Running the formatter on its own output must produce the same bytes. If not, file watchers (rust-analyzer, cargo build cache) churn forever.
2. **Error propagation.** Invalid Rust input must surface as `Err(TyrusError::FormattingError)`, not silently pass through. A historical `// formatting skipped for edition compatibility` marker once allowed unparseable code to reach disk and only fail at the next `cargo build` — losing the error attribution to codegen.
3. **No bypass marker.** That historical fallback is forbidden in output. Any future formatter swap must enforce the same.

PR #142 (`db391e6` — `test(orchestrator): add format_code regression gates for #131`) added unit tests for all three properties but did not commit a formal contract document. This ADR is that contract.

## Decision

`format_code(code: &str) -> Result<String, TyrusError>` in `crates/tyrus_orchestrator/src/format.rs` is **the** formatting touchpoint of the pipeline. Every emitted Rust file passes through it. The contract guarantees:

### Property 1 — Idempotence

For every `s: String` such that `format_code(&s)` returns `Ok(out)`, `format_code(&out)` returns `Ok(out)` (byte-equivalent). Equivalent: `format_code(format_code(s)?) == format_code(s)`.

**Implementation.** Currently delegated to `prettyplease::unparse(&syn::parse_file(code)?)`. `prettyplease` is documented-stable on this property. The unit test `format_code_is_idempotent` asserts it on representative source.

### Property 2 — Error propagation

Invalid Rust returns `Err(TyrusError::FormattingError(String))`, where the inner string is `syn::Error::to_string()`. The error is **not** swallowed, masked, or downgraded. Callers (`tyrus_orchestrator::pipeline::*`) propagate `Err` up to the CLI, which surfaces it as a build failure with the offending crate identified.

The unit test `format_code_propagates_parse_errors` asserts this on deliberately malformed input.

### Property 3 — No bypass marker

The string `// formatting skipped` (or any variant) **must not appear** in output. The historical `format_code` had a fallback that, on `syn::Error`, returned the input verbatim with this marker prepended. That fallback is removed. The unit test `format_code_handles_async_without_bypass` asserts that valid `async fn` source — which once tripped the fallback — now formats normally.

### Substitution clause

A future formatter swap (`rustfmt-nightly`, custom AST printer, etc.) is allowed only if the replacement preserves all three properties. The substitution must include:

- All three regression tests still pass byte-for-byte.
- `format_code` signature unchanged.
- Workspace `Cargo.toml` dependency updated atomically with the implementation.

A swap that breaks any property is a Rule 7 violation regardless of intent.

## Consequences

### Positive

- **Pipeline integrity.** The "compile failure" → "formatter masks it" → "build failure attributed to wrong crate" failure mode is architecturally closed. Errors surface at the layer that produced them.
- **Cache correctness.** Idempotent output keeps file mtimes stable across re-emissions, so Cargo's incremental compilation is not invalidated by pure-formatting churn.
- **Reviewable formatter swaps.** The contract gives a checklist for any future formatter substitution — the conversation is "does it preserve the three properties?" not "is the new formatter generally good?".

### Negative

- **Locks in `prettyplease` until tested otherwise.** Any formatter change has to clear the regression tests; that costs time. (Acceptable trade — the cost is bounded and the alternative is silent miscompile risk.)
- **Test fragility on edge cases.** The regression tests cover the three properties on representative source. They do not exhaust every possible `syn`-parseable Rust expression; new edge cases will be found and added over time.

## Alternatives rejected

- **Treat `format_code` as a library detail with no contract.** Was the pre-#142 state. Allowed PR #142's bugs to occur in the first place. Rejected.
- **Swap to subprocess `rustfmt`.** Adds an external-tool dependency, slows the pipeline by ~100× on small files, requires Rust toolchain in any deployment environment. Rejected.
- **Skip formatting in optimised builds and emit raw `quote!` output.** Loses readability of generated code; makes emitted `.rs` files unreviewable in PRs and unusable for users inspecting Tyrus output. Rejected.
- **Custom AST printer.** Would re-implement what `prettyplease` already does correctly. ~2,000 LOC of net-new code, perpetual maintenance, with zero behavioural improvement over `prettyplease`. Rejected.

## References

- Originating PR: [#142](https://github.com/Gefferson-Souza/Tyrus/pull/142) (`db391e6`) — `test(orchestrator): add format_code regression gates for #131`.
- Implementation: `crates/tyrus_orchestrator/src/format.rs`.
- Regression tests: `format::tests::format_code_is_idempotent`, `format::tests::format_code_propagates_parse_errors`, `format::tests::format_code_handles_async_without_bypass`.
- Related: Rule 7 (POWER_OF_TEN.md), ADR 0008.
