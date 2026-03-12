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

- **Input:** AST.
- **Rules:** Bans `any`, `eval`, and unassigned `var`.
- **Visitors:** `LintVisitor` (enforces rules) + `DecoratorVisitor` (extracts NestJS metadata).
- **Output:** Validated AST + Metadata.

### 3. Orchestration (`tyrus_orchestrator`)

The coordinator of the full pipeline.

- **Responsibility:** Manages multi-file resolution, project scoping, and generation of the Rust directory structure (`Cargo.toml`, `src/main.rs`).
- **Dependency Injection:** Resolves singleton patterns (NestJS Services) to `Arc<T>` / Axum `State` via `tyrus_di`.
- **Graph Resolution:** Uses `tyrus_di` to topologically sort dependencies and determine instantiation order.

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
| `tyrus_cli`         | CLI entry point (clap). Binary crate.                                 |
| `tyrus_parser`      | Wraps SWC parser. Input: `.ts` file. Output: `swc_ecma_ast::Program`. |
| `tyrus_ast`         | Reserved for future typed IR. SWC AST is used directly for now.       |
| `tyrus_analyzer`    | `LintVisitor` + `DecoratorVisitor`. Enforces the Oxidizable Standard. |
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
├── stmt.rs         — statement conversion (convert_stmt, convert_stmt_recursive)
├── fn_decl.rs      — function declaration processing (process_fn_decl)
├── class.rs        — class → struct+impl, Arc<Mutex<T>> state, NestJS controller/service patterns
├── module.rs       — module/import handling
├── type_mapper.rs  — TypeScript → Rust type mapping
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

Three visitors traverse the SWC AST via `swc_ecma_visit::Visit`:

1. `LintVisitor` — rejects `var`, `any`, `eval` (in `tyrus_analyzer`)
2. `DecoratorVisitor` — extracts `@Module`, `@Injectable`, `@Controller` metadata (in `tyrus_analyzer`)
3. `RustGenerator` — produces Rust token streams (entry point: `interface.rs`)

---

## Testing Architecture

Tests live in the `tests/` crate, organized by tier:

```
tests/
├── src/
│   ├── unit/          — unit tests for isolated functions
│   ├── snapshot/      — insta snapshot tests for codegen output
│   └── compilation/   — compilation tests: generate Rust, run rustc
└── fixtures/
    ├── tier1/         — core language features
    ├── tier2/         — advanced type system
    ├── tier3/         — ecosystem & async
    └── tier4/         — framework integration (NestJS → Axum)
```

Each fixture contains an `input.ts` verified against insta `.snap` files or compiled with `assert_rust_compiles()`.

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
