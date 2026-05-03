# Changelog

All notable changes to Tyrus (TypeScript → Rust transpiler) are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-05-02

### Added — Decorator Registry (Caminho C, ADR 0007)

- **New crate `tyrus_decorator_kinds`** — zero-dependency single source of truth for NestJS decorator name → `DecoratorKind` classification. Shared by `tyrus_analyzer` (DI graph extraction) and `tyrus_codegen` (handler dispatch) without dragging `proc_macro2` into the analyzer.
- **New module `tyrus_codegen::decorators`** with three handler traits (`ClassDecoratorHandler`, `MethodDecoratorHandler`, `ParamDecoratorHandler`), a `DecoratorRegistry` with O(1) HashMap lookup, and a process-wide singleton via `OnceLock`.
- **Class-level handlers** — `ControllerHandler`, `UseGuardsHandler`.
- **Method-level handlers** — `HttpMethodHandler` (parameterized by `DecoratorKind`, registered for all five HTTP verbs), `HttpCodeHandler` with input validation (rejects non-finite, negative, fractional, out-of-range values).
- **Param-level handlers** — `BodyHandler`, `ParamHandler`, `QueryHandler`, `HeadersHandler`.
- **`tier4_nestjs` integration tests** — registry isolation pinning, handler-kind invariants, multi-decorator combinations.
- **ADR 0007** — full architectural decision record for the registry pattern (`docs/architecture/decisions/0007-decorator-registry.md`).

### Added — Sprint 1 Quick Wins (PR #107)

- Compound assignment operators: `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`.
- `Map<K,V>` → `HashMap<K,V>`, `Set<T>` → `HashSet<T>` (including `new Map()` / `new Set()` constructors).
- `this.method()` recursive self-calls.
- Object shorthand `{name}` → `{name: name}`.
- `Date.now()` → `chrono::Utc::now().timestamp_millis()`.
- Object spread `{...base, field: v}` → struct update syntax.

### Added — Phase 8 (HTTP Equivalence E2E, PR #103)

- `test_http_equivalence_rust_server` — transpiles the reference NestJS project, compiles the generated Rust, starts both servers, and compares HTTP responses byte-for-byte.

### Changed

- **Replaced** runtime `unwrap_or` fallback in `map_status_code` with a static `STATUS_CODES: &[(u16, &str)]` table + `compile_error!` for unrecognized codes. Generated Rust no longer carries `unwrap_or` for status code mapping.
- **Eliminated** duplicate verb dispatch — the literal `"Get"` / `"Post"` / etc. is now matched in exactly one place (`DecoratorKind::from_name`).
- **Replaced** `Option<String>` round-trip in `convert_single_param` (NestJS param decorator dispatch) with strong-typed registry lookup.

### Deleted

- Heuristic `class_name.ends_with("Controller")` in `class/mod.rs` — replaced by decorator-driven detection (`ControllerHandler` flips `ControllerInfo.is_controller`).
- `find_param_decorator` helper (the stringly-typed `Option<String>` round-trip).
- `extract_single_decorator` — merged into the registry's `apply_method_decorators`.

### Fixed

- `@HttpCode` argument validation: `@HttpCode(70000)` no longer wraps silently to a different code; `@HttpCode(404.5)` is rejected at handler time. Out-of-range, fractional, NaN, and infinite values are dropped (equivalent to omitting the decorator).
- Test fixtures using `headers: any` (Oxidizable Standard violation) replaced with `Record<string, string>`.

### Stats

- **Tests:** 235 passing (1 skipped) — up from 211 baseline.
- **Crates:** 11 (added `tyrus_decorator_kinds`).
- **Codegen size:** ~6000 lines across 39 modules (was ~5040 / 32).
- **HTTP equivalence:** verified end-to-end against reference NestJS project.

## Earlier history

Prior to v0.1.0 (Sprint 1 milestone), the project went through Phases 1-8:

- **Phase 1-2** — Foundation + quality (strict lints, decomposition, test suite).
- **Phase 3** — Semantic equivalence (`assert_output_equivalent`).
- **Phase 4** — Control flow expansion (try-catch, switch, do-while, for, for-of).
- **Phase 5** — Stdlib coverage (16 string + 15 array + 15 math + 5 console + JSON + Object).
- **Phase 5.5** — Branded CLI, typed IR, expanded analyzer (7 lints + 11 API blocks).
- **Phase 6** — Class inheritance, static methods, getters/setters, type assertions, numeric enums, spread/rest, top-level statements.
- **Phase 7** — NestJS framework: @Param/@Query/@HttpCode/@UseGuards, HttpException → AppError, multi-module DI.
- **Phase 8** — HTTP equivalence E2E proven.

[Unreleased]: https://github.com/Gefferson-Souza/Tyrus/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Gefferson-Souza/Tyrus/releases/tag/v0.1.0
