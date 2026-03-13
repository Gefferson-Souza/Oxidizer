# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

**Tyrus** (TypeRust) is an academic TypeScript-to-Rust transpiler. It converts a strict subset of TypeScript ("Oxidizable Standard") into memory-safe Rust code. No `any`, `var`, or `eval` allowed. Cargo workspace with 10 crates under `crates/`.

## STRICT RULES (NEVER VIOLATE)

These rules are enforced by `.cargo/config.toml` and CI. Violations = compile error.

| Rule | Violation | Use Instead |
|------|-----------|-------------|
| No `.unwrap()` | `clippy::unwrap_used` | `?`, `.unwrap_or()`, `.unwrap_or_default()`, `.unwrap_or_else()`, `match` |
| No `.expect()` | `clippy::expect_used` | Same as unwrap alternatives |
| No `panic!()` | `clippy::panic` | `Result<T, TyrusError>` |
| No `todo!()` | `clippy::todo` | `compile_error!("Tyrus: ...")` in generated code, `Result::Err` in lib code |
| No string concat for codegen | Manual review | `quote!` macros only |
| Files < 400 lines | Code review | Split into modules |
| Functions < 50 lines | `clippy::too_many_lines` | Extract helpers |
| Max 5 function params | `clippy::too_many_arguments` | Create context structs |
| Max 4 nesting levels | `clippy::cognitive_complexity` | Early returns, extract functions |
| `pub(crate)` not `pub` | Code review | Only expose what other crates need |

## Build & Dev Commands

```bash
# Build
cargo build --workspace

# Tests (use nextest for speed)
cargo nextest run --workspace          # All tests (parallel)
cargo test -p integration_tests        # Integration tests only
cargo test -p tyrus_codegen            # Single crate

# Lint & Format (must pass CI)
cargo fmt -- --check
cargo clippy --workspace               # -Dwarnings is in .cargo/config.toml

# Run the compiler
cargo run --bin tyrus -- check <file.ts>              # Analyze for compatibility
cargo run --bin tyrus -- check --json <file.ts>       # JSON diagnostic output
cargo run --bin tyrus -- build <dir>/src --output <dir>/output   # Transpile to Rust
cargo run --bin tyrus -- compile <dir>/src --output <dir>/output # Transpile + cargo build
cargo run --bin tyrus -- run <dir>/src --output <dir>/output     # Transpile + build + execute
cargo run --bin tyrus -- --quiet <command>             # Suppress banner

# Snapshots
cargo insta review                     # Review snapshot changes
```

## Architecture

```
.ts input
  → tyrus_parser       (SWC parsing → swc_ecma_ast::Program)
  → tyrus_ast           (TyrusModule IR — typed intermediate representation)
  → tyrus_analyzer     (LintVisitor + DecoratorVisitor + UnsupportedApiVisitor)
  → tyrus_di           (petgraph topological sort for DI)
  → tyrus_orchestrator (multi-file coordination, CheckResult)
  → tyrus_codegen      (quote! → proc_macro2::TokenStream)
  → formatted .rs output
```

### Crate Map

| Crate | Lines | Role |
|-------|-------|------|
| `tyrus_cli` | ~80 | CLI (clap). 4 commands: `check`/`build`/`compile`/`run`. Branded output. |
| `tyrus_parser` | ~55 | Wraps SWC parser. `.ts` → `Program` |
| `tyrus_ast` | ~400 | Typed IR: `TyrusModule`/`TyrusExpr`/`TyrusStmt`/`TyrusDecl`. SWC→IR lowering. |
| `tyrus_analyzer` | ~450 | `LintVisitor` (8 rules) + `DecoratorVisitor` + `UnsupportedApiVisitor` + JSON reports |
| `tyrus_codegen` | ~2540 | **Core.** `RustGenerator` → TokenStream. Decomposed: `helpers/stmt/fn_decl/expr/*/class/*`. |
| `tyrus_di` | ~195 | DI graph (petgraph). Topological sort. |
| `tyrus_orchestrator` | ~527 | Pipeline coordination. Split: `lib/pipeline/scaffold/format`. |
| `tyrus_diagnostics` | ~69 | `TyrusError` + miette |
| `tyrus_common` | ~70 | `FilePath`, `to_snake_case()`, config |
| `tyrus_test_utils` | ~86 | `assert_rust_compiles()` (allows unwrap in tests) |

### Codegen Module Structure (Current)

```
crates/tyrus_codegen/src/
├── convert/
│   ├── mod.rs            # Module declarations
│   ├── interface.rs      # RustGenerator visitor, interfaces → structs
│   ├── helpers.rs        # to_snake_case, to_pascal_case, is_string_expr, is_primitive_type
│   ├── fn_decl.rs        # Function declaration transpilation
│   ├── stmt.rs           # Statement conversion
│   ├── type_mapper.rs    # TS→Rust type mapping
│   └── expr/             # Expression conversion
│       ├── mod.rs         # Expression dispatcher
│       ├── binary.rs      # Binary operators (+, -, *, /, ==, etc.)
│       ├── call.rs        # Function/method calls
│       ├── member.rs      # Member access (obj.field)
│       ├── arrow.rs       # Arrow functions
│       ├── literal.rs     # Object/array/template literals
│       └── misc.rs        # Assignment, update, unary, optional chaining
│   ├── class/            # Class → struct+impl (split from 1048-line monolith)
│   │   ├── mod.rs         # Class dispatcher + property conversion
│   │   ├── constructor.rs # Constructor transpilation + DI
│   │   ├── method.rs      # Method transpilation + decorators
│   │   ├── routing.rs     # Axum router generation + FromRequestParts
│   │   └── mutation.rs    # Self-mutation detection
│   └── module.rs         # Module/import handling
└── stdlib/               # Standard library mappings
    ├── mod.rs, console.rs, array.rs, string.rs, math.rs, json.rs
```

## Type Mappings (TS → Rust)

| TypeScript | Rust |
|------------|------|
| `string` | `String` |
| `number` | `f64` |
| `boolean` | `bool` |
| `void` | `()` |
| `T[]` / `Array<T>` | `Vec<T>` |
| `Promise<T>` | `Result<T, AppError>` |
| `Record<K,V>` | `HashMap<K,V>` |
| `T \| undefined` | `Option<T>` |
| `interface` | `#[derive(Serialize, Deserialize)] struct` |
| `type Status = "a" \| "b"` | `enum Status { A, B }` |
| `class` | `struct + impl` |

## Testing Architecture

```
tests/
├── src/
│   ├── cli.rs         # CLI integration: help, version, check, build, --json, --quiet
│   ├── unit/          # FAST (<5s): Test codegen functions directly
│   ├── snapshot/      # MEDIUM (<10s): Full transpilation → insta snapshots
│   ├── compilation/   # SLOW (<60s): Batch cargo check per tier
│   └── equivalence/   # SEMANTIC: Run TS (Node) + Rust, compare stdout
│       ├── basic.rs    # Arithmetic, control flow, unary ops
│       ├── strings.rs  # String methods equivalence
│       ├── arrays.rs   # Array methods equivalence
│       └── console.rs  # console.log formatting
├── fixtures/
│   ├── tier1/         # Basic: variables, math, functions, control flow
│   ├── tier2/         # Intermediate: interfaces, classes, async
│   ├── tier3/         # Advanced: generics, optional chaining, destructuring
│   └── tier4/         # NestJS: @Injectable, @Controller, full project
```

## Conventions

- **Commits:** `<type>: <description>` — Types: feat, fix, refactor, test, chore, docs, perf, ci
- **Branches:** `feat/`, `fix/`, `chore/`, `refactor/` prefixes
- **Error handling:** Always `Result<T, TyrusError>`. Never unwrap/expect/panic.
- **Generated code errors:** Use `compile_error!("Tyrus: ...")` instead of `todo!()`
- **Immutability:** Prefer `&self`, use `&mut self` only when mutation is detected
- **Code gen:** `quote!` macros only. Never string concatenation.

## Active Refactoring

See `docs/superpowers/plans/2026-03-12-full-refactoring-roadmap.md` for the complete plan.

**Completed:** Chunks 1-7 + Milestones 13A/13B + Milestone 14 HIGH + CLI/IR/Analyzer evolution
**Current:** Phase 5.5 complete — CLI branded (4 commands), IR defined (TyrusModule), Analyzer expanded (8 rules + unsupported API + JSON)
**Status:** 153 tests passing (51 equivalence + 8 IR + 7 CLI + 73 integration + 9 codegen + 4 common + 1 skipped)
