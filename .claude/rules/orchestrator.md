---
paths:
  - "crates/tyrus_orchestrator/**"
---

# Orchestrator Rules (binding — ADRs 0009/0010)

- **Formatter contract (ADR 0010):** `format::format_code` guarantees idempotence,
  error propagation (`TyrusError::FormattingError` — never silent passthrough), and
  no bypass marker in output. Any change to formatting must keep the three property
  tests green; swapping the formatter requires an ADR.
- `format.rs` is the workspace's ONLY permitted string-Rust touchpoint (R7 exception)
  — pre-validated `&'static str` blocks (`get_app_error_*`) and layout only. Do not
  grow that exception.
- **Mutex protocol (ADR 0009):** generated `@Injectable` state uses block-scoped
  reads and read-then-write splits for compound assignments; every generated `.lock()`
  uses `.unwrap_or_else(|e| e.into_inner())`. Orchestrator/scaffold changes must not
  produce code paths holding two guards at once.
- Scaffold output must be DETERMINISTIC: never iterate a `HashMap` directly into
  emitted code — sort or use the DI topological order. (A nondeterministic fallback
  in `generate_main_rs` was a known defect class here.)
- Analyzer errors during `build_project` currently print warnings instead of failing
  the build — that is bug #188 (UAT CRITICAL), not a pattern to imitate. New pipeline
  code fails fast on analyzer errors.
