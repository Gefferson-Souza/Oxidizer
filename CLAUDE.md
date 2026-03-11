# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tyrus (TypeRust) is an academic TypeScript-to-Rust compiler. It transpiles a strict subset of TypeScript ("Oxidizable Standard") into memory-safe Rust code with formal semantic preservation. No `any`, `var`, or `eval` are allowed. The project uses a Cargo workspace with 10 crates under `crates/`.

## Build & Development Commands

```bash
# Build
cargo build --workspace              # Debug build (all crates)
cargo build --release                # Release build with LTO

# Run the compiler
cargo run --bin tyrus -- check <file.ts>                        # Lint/analyze
cargo run --bin tyrus -- build <dir>/src --output <dir>/output  # Transpile project

# Tests
cargo test --workspace               # All tests
cargo test -p tests                   # Integration/snapshot tests only
cargo test -p tyrus_codegen           # Single crate tests
cargo test test_snapshot_             # Run tests matching a pattern

# Lint & Format (must pass CI)
cargo fmt -- --check
cargo clippy --workspace -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic
```

## CI Requirements

CI runs format check, clippy (with `unwrap_used`, `expect_used`, `panic` as warnings promoted to errors), build, tests, and compiles the `examples/real_world_demo/` output to verify end-to-end correctness. All code must be panic-free — use `Result<T, TyrusError>` instead.

## Architecture: Compilation Pipeline

The compiler is a multi-pass pipeline, each stage in its own crate:

```
.ts input
  → tyrus_parser       (SWC-based lexing/parsing → swc_ecma_ast::Program)
  → tyrus_analyzer      (lint validation + decorator extraction + DI graph)
  → tyrus_di            (dependency injection resolution via petgraph topological sort)
  → tyrus_orchestrator  (multi-file coordination, module wiring, ordering)
  → tyrus_codegen       (AST → Rust TokenStream via quote! macros)
  → formatted .rs output
```

### Crate Responsibilities

| Crate | Role |
|---|---|
| `tyrus_cli` | CLI entry point (clap). Binary crate. |
| `tyrus_parser` | Wraps SWC parser. Input: `.ts` file → Output: `swc_ecma_ast::Program` |
| `tyrus_ast` | Internal AST type definitions |
| `tyrus_analyzer` | `LintVisitor` (enforces Oxidizable Standard), `DecoratorVisitor` (extracts NestJS metadata) |
| `tyrus_codegen` | Core transpilation. `RustGenerator` visitor converts TS AST → `proc_macro2::TokenStream` |
| `tyrus_di` | NestJS-style dependency injection engine. Uses `petgraph::DiGraph` for topological sort. |
| `tyrus_orchestrator` | Coordinates the full pipeline: parse → analyze → codegen. Handles multi-file projects. |
| `tyrus_diagnostics` | `TyrusError` variants with `miette` integration for rich error reporting |
| `tyrus_common` | Shared types (`FilePath` newtype), config, filesystem utilities |
| `tyrus_test_utils` | Test helpers including `assert_rust_compiles()` |

### Key Code Generation Files

- `crates/tyrus_codegen/src/convert/interface.rs` — `RustGenerator`: main visitor, interfaces → structs, type aliases → enums
- `crates/tyrus_codegen/src/convert/func.rs` — Function transpilation, array/string method mapping, expression conversion
- `crates/tyrus_codegen/src/convert/class.rs` — Class → struct+impl, state management with `Arc<Mutex<T>>`, NestJS controller/service patterns

## Key Transpilation Patterns

- **Types:** `string→String`, `number→f64`, `boolean→bool`, `Promise<T>→Result<T, AppError>`, `Record<K,V>→HashMap<K,V>`
- **Interfaces** → `#[derive(Serialize, Deserialize)] struct` with serde
- **String union types** (`type Status = "open" | "closed"`) → Rust enums
- **Array methods** (`.map`, `.filter`, `.forEach`, `.find`, `.some`, `.every`) → iterator chains with `.collect()`. Supports `(item, index)` callbacks via `.enumerate()`
- **String methods** (`.includes→.contains`, `.replace→.replacen`, `.split→.split().collect()`, etc.)
- **Classes** → structs with `impl` blocks. State fields use `Arc<Mutex<T>>` for interior mutability. Constructor-injected deps wrapped in `Arc<T>`.
- **NestJS decorators** → Axum: `@Controller("/path")` → router, `@Get()` → `axum::routing::get`, `@Injectable()` → DI registration
- **Async/await** → `pub async fn` with tokio, `await` → `.await`

## Code Generation Approach

All Rust code is generated using `quote!` macros producing `proc_macro2::TokenStream` — never string concatenation. This ensures hygienic, injection-free code generation.

## Visitor Pattern

Three visitors traverse the SWC AST via `swc_ecma_visit::Visit`:
1. `LintVisitor` — rejects `var`, `any`, `eval`
2. `DecoratorVisitor` — extracts `@Module`, `@Injectable`, `@Controller` metadata
3. `RustGenerator` — produces Rust token streams

## Testing

- **Snapshot tests** (insta): fixtures in `tests/fixtures/*/input.ts`, verified against `.snap` files
- **Build integration tests**: run tyrus CLI via `assert_cmd`, verify generated Rust compiles
- **Compilation tests**: `assert_rust_compiles()` helper runs `rustc` on generated output

## Conventions

- **Commit format:** `<type>(<scope>): <subject>` — Conventional Commits (see CONTRIBUTING.md)
- **Branching:** `feat/`, `fix/`, `chore/`, `refactor/` prefixes. `main` is protected.
- **Error handling:** Always `Result<T, TyrusError>`. Never `unwrap()`, `expect()`, or `panic!()`.
- **Guidelines.md** contains detailed engineering guidelines (in Portuguese).
