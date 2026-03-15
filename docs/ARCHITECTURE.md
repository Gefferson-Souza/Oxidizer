# Tyrus Architecture

Tyrus is structured as a multi-stage compilation pipeline, designed for modularity and extensibility. This document describes the flow from TypeScript source to a final, runnable Rust project.

## The Compilation Pipeline

### 1. Parsing (`tyrus_parser`)

The entry point uses the `swc_ecma_parser` to ingest TypeScript source files.

- **Input:** `.ts` source code.
- **Output:** Abstract Syntax Tree (`swc_ecma_ast::Program`).
- **Responsibility:** Ensure the input is syntactically valid TypeScript.

### 2. Semantic Analysis (`tyrus_analyzer`)

This stage validates the AST against the **Oxidizable Standard**.

- **Input:** AST + source code + file name.
- **Lint rules (8):** Bans `var`, `any`, `eval`, `for-in`, `delete`, `with`, labeled statements. (Note: `try-catch` was unblocked in Phase 6.1)
- **Unsupported API detection (11):** Blocks `document`, `window`, `navigator`, `localStorage`, `sessionStorage`, `XMLHttpRequest`, `require`, `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`.
- **Visitors:** `LintVisitor` (8 rules) + `DecoratorVisitor` (NestJS metadata) + `UnsupportedApiVisitor` (blocked APIs).
- **Output:** `AnalysisResult { errors, diagnostics, graph }` — errors are `TyrusError`, diagnostics are structured `Diagnostic` with severity/code/span/suggestion.
- **Reports:** `format_pretty()` for colored terminal output, `format_json()` for tooling integration.

### 3. Orchestration (`tyrus_orchestrator`)

The coordinator of the full pipeline, decomposed into focused modules:

- **`lib.rs`** — Public API: `check()`, `build()`, `build_project()`, `build_simple_project()`
- **`pipeline.rs`** — Core multi-file build orchestration (walk/parse, analyze, DI graph, transpile, mod.rs)
- **`scaffold.rs`** — Project scaffolding: `generate_main_rs()`, `generate_cargo_toml()`, `generate_mod_rs()`
- **`format.rs`** — Code formatting and `AppError` code generation

Responsibilities:
- Manages multi-file resolution, project scoping, and Rust project structure generation.
- Resolves singleton patterns (NestJS Services) to `Arc<T>` / Axum `State` via `tyrus_di`.
- Uses `tyrus_di` to topologically sort dependencies and determine instantiation order.

### 4. Dependency Management (`tyrus_di`)

A dedicated crate for handling the application's dependency graph.

- **Input:** Module metadata and provider definitions.
- **Algorithm:** Topological sort via `petgraph`.
- **Output:** Ordered initialization list, separation of Modules vs Providers vs Controllers.

### 5. Code Generation (`tyrus_codegen`)

- **Responsibility:** Converts the validated AST to Rust `TokenStream` using `quote!` macros — never string concatenation.
- **Output:** Formatted Rust code written to `src/`.

---

## Crate Breakdown

| Crate               | Responsibility                                                        |
| :------------------ | :-------------------------------------------------------------------- |
| `tyrus_cli`         | CLI (clap). 4 commands: `check`/`build`/`compile`/`run`. Branded banner, progress pipeline, `--quiet`/`--json` flags. Installable globally via `cargo install --path crates/tyrus_cli`. |
| `tyrus_parser`      | Wraps SWC parser. Input: `.ts` file. Output: `swc_ecma_ast::Program`. |
| `tyrus_ast`         | Typed IR: `TyrusModule`, `TyrusExpr`, `TyrusStmt`, `TyrusDecl`, `TyrusType`. SWC→IR type lowering (`lower_type.rs`). |
| `tyrus_analyzer`    | `LintVisitor` (8 rules) + `DecoratorVisitor` + `UnsupportedApiVisitor` (11 APIs). Structured `Diagnostic` output with JSON/pretty formatters. |
| `tyrus_codegen`     | Core transpilation. `RustGenerator` visitor converts TS AST to Rust.  |
| `tyrus_di`          | NestJS-style DI engine. Uses `petgraph::DiGraph` for topo sort.       |
| `tyrus_orchestrator`| Coordinates the full pipeline: parse → analyze → codegen.            |
| `tyrus_diagnostics` | `TyrusError` variants with `miette` for rich error reporting.         |
| `tyrus_common`      | Shared types (`FilePath` newtype), config, filesystem utilities.       |
| `tyrus_test_utils`  | Test helpers including `assert_rust_compiles()`.                      |

---

## Code Generation Module Structure (`tyrus_codegen`)

The `crates/tyrus_codegen/src/convert/` directory is decomposed into focused, single-responsibility modules:

```
convert/
├── mod.rs          — module declarations and re-exports
├── interface.rs    — RustGenerator struct definition + Visit impl (pipeline entry point)
├── helpers.rs      — shared utilities: to_snake_case, to_pascal_case, is_string_expr
├── fn_decl.rs      — function declaration processing (process_fn_decl)
├── module.rs       — module/import handling
├── type_mapper.rs  — TypeScript → Rust type mapping (deduplicated map_type_core)
├── stmt/           — statement conversion
│   ├── mod.rs          — dispatcher + convert_stmt, convert_stmt_recursive
│   ├── var_decl.rs     — variable declarations (ident, object/array destructuring)
│   ├── loops.rs        — while, for-of, for-in, for, do-while
│   ├── switch.rs       — switch → match
│   └── try_catch.rs    — try-catch → Result matching
├── class/          — class → struct+impl
│   ├── mod.rs          — dispatcher + property conversion
│   ├── constructor.rs  — constructor transpilation + DI
│   ├── method.rs       — method transpilation + decorators
│   ├── routing.rs      — Axum router generation + @UseGuards middleware
│   └── mutation.rs     — self-mutation detection
└── expr/
    ├── mod.rs      — expression dispatcher (convert_expr)
    ├── binary.rs   — binary operators (convert_bin_expr)
    ├── call.rs     — function/method calls, axios/fetch/array methods (map/filter/forEach/find)
    ├── member.rs   — property access, mutex state (convert_member_expr)
    ├── arrow.rs    — arrow functions → closures (convert_arrow_expr)
    ├── literal.rs  — literals, object/array/template expressions
    └── misc.rs     — assignments, updates, optional chaining
```

### Key Transpilation Patterns

- **Types:** `string→String`, `number→f64`, `boolean→bool`, `Promise<T>→Result<T, AppError>`, `Record<K,V>→HashMap<K,V>`
- **Interfaces** → `#[derive(Serialize, Deserialize)] struct` with serde
- **String union types** (`type Status = "open" | "closed"`) → Rust enums
- **Array methods** (`.map`, `.filter`, `.forEach`, `.find`, `.some`, `.every`) → iterator chains with `.collect()`. Supports `(item, index)` callbacks via `.enumerate()`
- **String methods** (`.includes→.contains`, `.replace→.replacen`, `.split→.split().collect()`, etc.)
- **Classes** → structs with `impl` blocks. State fields use `Arc<Mutex<T>>` for interior mutability. Constructor-injected deps wrapped in `Arc<T>`.
- **NestJS decorators** → Axum: `@Controller("/path")` → router, `@Get()` → `axum::routing::get`, `@Injectable()` → DI registration
- **Async/await** → `pub async fn` with tokio, `await` → `.await`

---

## Visitor Pattern

Four visitors traverse the SWC AST via `swc_ecma_visit::Visit`:

1. `LintVisitor` — rejects `var`, `any`, `eval`, `for-in`, `delete`, `with`, labeled (in `tyrus_analyzer`)
2. `DecoratorVisitor` — extracts `@Module`, `@Injectable`, `@Controller` metadata (in `tyrus_analyzer`)
3. `UnsupportedApiVisitor` — detects DOM, browser, timer, CommonJS APIs (in `tyrus_analyzer`)
4. `RustGenerator` — produces Rust token streams (entry point: `interface.rs`)

---

## Testing Architecture

Tests live in the `tests/` crate, organized by tier:

```
tests/
├── src/
│   ├── cli.rs         — CLI integration tests (help, version, check, build, flags)
│   ├── unit/          — unit tests for isolated functions
│   ├── snapshot/      — insta snapshot tests for codegen output
│   ├── compilation/   — compilation tests: generate Rust, run rustc
│   └── equivalence/   — semantic equivalence: TS and Rust produce identical output
└── fixtures/
    ├── tier1/         — core language features
    ├── tier2/         — advanced type system
    ├── tier3/         — ecosystem & async
    └── tier4/         — framework integration (NestJS → Axum)
```

179 tests across 7 categories: equivalence (71), unit (49), snapshot (20), IR (21), compilation (9), CLI (7), trybuild (1).

---

## Strict Quality Rules

The project enforces strict correctness via:

- **`.cargo/config.toml`:** `-Dwarnings` plus `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::todo`, `clippy::unimplemented` as hard errors.
- **`clippy.toml`:** Cognitive complexity threshold 15, function lines threshold 50, max 5 parameters.
- **`deny.toml`:** Dependency license and security audit via `cargo-deny`.
- **CI:** GitHub Actions with `actions/checkout@v4`, `Swatinem/rust-cache@v2`, `cargo nextest`, and end-to-end demo compilation verification.

---

## Tech Stack

- **Source Language:** TypeScript (via SWC)
- **Target Language:** Rust (1.75+)
- **Macro Engine:** `quote`, `proc_macro2`
- **Internal Web Engine:** `axum` 0.7 (for NestJS mappings)
- **Test Runner:** `cargo nextest` + `insta` (snapshot testing)
