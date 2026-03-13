# Contributing to Tyrus

## 🌳 Git Workflow: The "Tyrus Pattern"

We follow a strict **Feature Branch Workflow** combined with **Conventional Commits**.

### Branching Strategy

- **`main`**: Protected. Production-ready code only. No direct commits.
- **`feat/`**: New features (e.g., `feat/async-await`, `feat/new-parser`).
- **`fix/`**: Bug fixes (e.g., `fix/memory-leak`, `fix/cli-panic`).
- **`chore/`**: Maintenance, config, docs (e.g., `chore/optimize-workflow`, `docs/update-readme`).
- **`refactor/`**: Code restructuring without behavior change.

### 📝 Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/).

**Format:** `<type>(<scope>): <subject>`

**Types:**

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools and libraries such as documentation generation

**Examples:**

- `feat(codegen): implement structural typing for interfaces`
- `fix(cli): resolve panic when input file is missing`
- `chore(deps): upgrade axum to v0.7`

## 🚀 Pull Request Process

1.  Create a branch complying with the strategy above.
2.  Ensure tests pass locally: `cargo nextest run --workspace`
3.  Run the full clippy suite and fix all warnings:
    ```bash
    cargo clippy --workspace -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::panic
    ```
4.  Run formatter: `cargo fmt -- --check`
5.  Open a PR to `main`.
6.  Fill out the **PR Template** completely.
7.  Wait for CI checks to pass and request review.

## 🧪 Test Infrastructure

Tests live in the `tests/` crate and are organized by tier:

```
tests/src/
├── unit/          — unit tests for isolated functions/utilities
├── snapshot/      — insta snapshot tests (codegen output verification)
└── compilation/   — compilation tests (generate Rust, invoke rustc)
```

### Running specific test types

```bash
# All tests (preferred: uses nextest)
cargo nextest run --workspace

# Legacy test runner
cargo test --workspace

# Only snapshot tests (insta)
cargo test -p tests snapshot

# Only compilation tests
cargo test -p tests compilation

# Update snapshots after intentional codegen changes
cargo insta review
```

### Strict code rules (enforced by CI)

- **Never** use `.unwrap()`, `.expect()`, or `panic!()` — use `?` and `Result<T, TyrusError>`.
- **Never** use `todo!()` or `unimplemented!()` — use `compile_error!()` or proper error variants.
- **Never** use string concatenation for code generation — use `quote!` macros.
- Functions must stay under 50 lines; cognitive complexity threshold is 15.
- All new code must be covered by at least one test.

## 🏗 Code Generation Module Map

When working on code generation, the relevant files are under `crates/tyrus_codegen/src/convert/`:

| Module | Responsibility |
|---|---|
| `interface.rs` | `RustGenerator` struct + `Visit` impl (pipeline entry point) |
| `helpers.rs` | `to_snake_case`, `to_pascal_case`, `is_string_expr` |
| `stmt.rs` | Statement conversion |
| `fn_decl.rs` | Function declaration processing |
| `module.rs` | Module/import handling |
| `type_mapper.rs` | TypeScript → Rust type mapping (`map_type_core`) |
| **`class/`** | **Class → struct+impl (decomposed from monolithic `class.rs`)** |
| `class/mod.rs` | Class dispatcher + property conversion |
| `class/constructor.rs` | Constructor transpilation + DI detection |
| `class/method.rs` | Method transpilation + decorator parsing |
| `class/routing.rs` | Axum router generation + `FromRequestParts` |
| `class/mutation.rs` | Static self-mutation analysis |
| **`expr/`** | **Expression conversion (decomposed from monolithic `func.rs`)** |
| `expr/mod.rs` | Expression dispatcher |
| `expr/call.rs` | Function/method calls, array methods |
| `expr/member.rs` | Property access, mutex state |
| `expr/binary.rs` | Binary operators |
| `expr/arrow.rs` | Arrow functions → closures |
| `expr/literal.rs` | Literals, objects, arrays, template literals |
| `expr/misc.rs` | Assignments, updates, optional chaining |

### Orchestrator Module Map

The orchestrator (`crates/tyrus_orchestrator/src/`) coordinates multi-file builds:

| Module | Responsibility |
|---|---|
| `lib.rs` | Slim public API (`build()` entry point) |
| `pipeline.rs` | Core multi-file build orchestration |
| `scaffold.rs` | Project scaffolding (`main.rs`, `Cargo.toml`, `mod.rs`) |
| `format.rs` | Code formatting + `AppError` generation |
