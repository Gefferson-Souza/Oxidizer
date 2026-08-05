---
name: new-decorator
description: Add support for a NestJS decorator via the registry (R8/ADR 0007) — the one-file-plus-two-lines path. Use whenever a new @Decorator must be recognized.
---

# /new-decorator — the R8-compliant path (nothing else is acceptable)

Adding decorator `@X` touches EXACTLY:

1. `crates/tyrus_decorator_kinds/src/lib.rs` — new `DecoratorKind` variant (documented
   — the crate denies missing_docs) + arm in `from_name` + scope in `scope()`.
2. `crates/tyrus_codegen/src/decorators/<x>.rs` — ONE handler file implementing the
   trait for its scope (`ClassDecoratorHandler` / `MethodDecoratorHandler` /
   `ParamDecoratorHandler`).
3. `crates/tyrus_codegen/src/decorators/mod.rs` — ONE `register_*` line in
   `default_registry()`.
4. Tests: equivalence test (F2, RED first) exercising the decorator end-to-end, plus
   a handler isolation unit test.

**Forbidden:** touching `convert/class/*` or `convert/interface.rs` for dispatch,
raw string comparison of decorator names, `match name { "X" => … }` anywhere. If the
decorator seems to need more than the four touchpoints above, STOP — that's an R8
violation or a registry gap; open an issue/ADR instead of hacking around it.

Precedent: ADR 0007 (`docs/architecture/decisions/0007-decorator-registry.md`);
`@Headers` in `decorators/params.rs` is the reference implementation.

$ARGUMENTS: decorator name + semantics (e.g. "@Redirect(url, status)").
