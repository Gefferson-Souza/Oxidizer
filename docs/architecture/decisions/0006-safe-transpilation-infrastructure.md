# 6. Safe Transpilation Infrastructure

Date: 2026-03-12
Status: Accepted

## Context

The compiler codebase previously relied on `todo!()`, `.expect()`, and `.unwrap()` for error handling. These panic at runtime, making the compiler unpredictable when encountering unsupported TypeScript constructs. A transpiler that crashes instead of producing a meaningful error is unacceptable for an academic tool.

Additionally, there was no formal enforcement of code quality thresholds, no dependency audit, and the CI pipeline used outdated GitHub Actions.

## Decision

### Strict Linting (Compile-Time Enforcement)

Added `.cargo/config.toml` with `-Dwarnings` and the following clippy lints as **hard errors**:

- `clippy::unwrap_used` — No `.unwrap()` in production paths
- `clippy::expect_used` — No `.expect()` in production paths
- `clippy::panic` — No `panic!()` macros
- `clippy::todo` — No `todo!()` stubs
- `clippy::unimplemented` — No `unimplemented!()` stubs

### Quality Thresholds

Added `clippy.toml` with:

- Cognitive complexity threshold: 15
- Function lines threshold: 50
- Maximum function parameters: 5

### Dependency Audit

Added `deny.toml` for `cargo-deny`:

- License allowlist (MIT, Apache-2.0, BSD variants)
- Security advisory database checks

### Error Handling Strategy

| Before | After |
|--------|-------|
| `todo!()` in generated code | `compile_error!("Tyrus: ...")` |
| `.expect("msg")` in lib code | `?` operator with `Result<T, TyrusError>` |
| `.unwrap()` everywhere | `.unwrap_or()`, `.unwrap_or_default()`, `match` |
| `panic!("unsupported")` | `Result::Err(TyrusError::...)` |

### CI Modernization

- GitHub Actions `v4` (`actions/checkout@v4`)
- `Swatinem/rust-cache@v2` for build caching
- `cargo nextest` for parallel test execution
- End-to-end demo compilation verification in CI

## Consequences

### Positive

- Compiler never panics on valid or invalid input — always produces a meaningful error or `compile_error!()`.
- Code quality is enforced at compile time, not just by code review.
- Dependencies are audited for known vulnerabilities and license compliance.
- CI runs faster (~5min vs ~9min) with caching and parallel checks.

### Negative

- Stricter rules increase development friction (every new function must handle errors properly).
- `compile_error!()` in generated code produces less informative messages than a full diagnostic.
- `cargo-deny` may flag transitive dependencies that are difficult to replace.
