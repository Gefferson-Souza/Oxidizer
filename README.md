> 🌐 **Language:** English | [Português (BR)](README.pt-br.md)

# Tyrus: A High-Fidelity TypeScript-to-Rust Compiler

_Academic Project in Compiler Theory & Semantic Preservation_

[![CI Status](https://github.com/Gefferson-Souza/Tyrus/actions/workflows/ci.yml/badge.svg)](https://github.com/Gefferson-Souza/Tyrus/actions/workflows/ci.yml)
![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

Tyrus is a source-to-source compiler designed to bridge the gap between high-level dynamic syntax (TypeScript) and low-level memory safety (Rust). As an academic initiative, it focuses on the formal mapping of higher-order abstractions to zero-cost Rust equivalents, exploring the boundaries of **Semantic Preservation** across differing execution models.

---

## 🔬 Core Philosophies

### 🛡️ Zero Magic

Tyrus does not rely on a hidden runtime or garbage collection emulation. Every TypeScript construct is mapped to its most efficient Rust equivalent, leveraging Rust's ownership model and strict type system at compile time.

### 📐 Semantic Preservation

The primary goal is formal equivalence. If a TypeScript project is "Oxidizable," the generated Rust code is guaranteed to maintain the original logic's semantic integrity while improving performance and security.

### 🚫 The Oxidizable Standard

Tyrus enforces a strict subset of TypeScript called the "Oxidizable Standard." It rejects non-idiomatic or unsafe patterns (like `any` or `eval`) to ensure the resulting Rust code is both safe and performant.

### 🔐 Safe Transpilation Architecture

Adhering to strict "Safe Transpilation" principles:

- **Panic-Free Compilation**: Compiler logic uses robust error handling instead of panicking on invalid input.
- **Strict Linting**: The codebase is verified with `clippy::pedantic` rules (e.g., no `unwrap()`/`expect()` in production paths).
- **Formal AST Mapping**: Uses Algebraic Data Types (ADTs) to represent logic, avoiding string manipulation vulnerabilities.

---

## 🚀 Feature Tiers

### Tier 1: Core Language (Production Ready)

- Primitives (`string`, `number`, `boolean`)
- Control Flow (`if/else`, `while`, `for`)
- Error Handling (`Result`, `Option`)

### Tier 2: Advanced Type System (Production Ready)

- Interfaces and Type Aliases to Structs/Enums
- Generics and Polymorphism
- Comprehensive Collection Mapping (`Array<T>` -> `Vec<T>`)

### Tier 3: Ecosystem & Asynchony (Production Ready)

- `Async/Await` to Future-based concurrency
- JSON Serialization/Deserialization (via `serde`)
- HTTP Client and REST patterns (via `axum` & `reqwest`)

### 📦 Supported Patterns (Verified with Semantic Equivalence Tests)

- **String Methods** (16): `includes`, `replace`, `split`, `toUpperCase`, `toLowerCase`, `trim`, `startsWith`, `endsWith`, `toString`, `substring`, `charAt`, `indexOf`, `repeat`, `slice`, `padStart`, `padEnd`
- **Array Methods** (15): `map`, `filter`, `forEach`, `find`, `some`, `every`, `reduce`/`fold`, `join`, `includes`, `push`, `indexOf`, `slice`, `concat`, `reverse`, `pop`, `sort`, `shift`, `flat`, `flatMap`
- **Math Functions** (15): `max`, `min`, `round`, `floor`, `ceil`, `abs`, `random`, `pow`, `sqrt`, `log`, `trunc`, `sign`, `sin`, `cos`, `tan`
- **Math Constants** (2): `Math.PI`, `Math.E`
- **Console** (5): `log`, `error`, `warn`, `info`, `debug`
- **Control Flow**: `if/else`, `while`, `for`, `for-of`, `do-while`, `switch/case`, ternary
- **Operators**: Arithmetic, comparison, logical, `**` (exponentiation), `%` (modulo)
- **Class State**: Automatic `Arc<Mutex<T>>` wrapping for services/controllers
- **Interfaces**: `interface` -> `#[derive(Serialize, Deserialize)] struct`
- **String Unions**: `type Status = "a" | "b"` -> `enum` with `Display` and `PartialEq`

---

## 🛠 Installation & Usage

### Prerequisites

- Rust 1.75+ (Stable)
- Cargo

### Setup

```bash
git clone https://github.com/Gefferson-Souza/Tyrus.git
cd Tyrus
cargo build --release
```

### Install Globally

```bash
# Install tyrus as a global CLI command
cargo install --path crates/tyrus_cli

# Now use from anywhere
tyrus --version
```

### Using the Compiler

```bash
# Analyze a TypeScript file for compatibility
tyrus check ./src/index.ts

# Get JSON diagnostics (for tooling integration)
tyrus check --json ./src/index.ts

# Transpile to Rust source code (stdout or file)
tyrus build ./src/index.ts
tyrus build ./src/index.ts -o output.rs

# Transpile + compile to native binary
tyrus compile ./src/index.ts --output ./output

# Transpile + compile + execute
tyrus run ./src/index.ts --output ./output

# Suppress banner for scripting
tyrus --quiet check ./src/index.ts
```

---

## 📋 Commands Reference

<!-- AUTO-GENERATED from Cargo.toml and CI -->

**Development Commands:**

| Command | Description |
|---------|-------------|
| `cargo build --workspace` | Build all workspace crates |
| `cargo build --release` | Production build with LTO |
| `cargo install --path crates/tyrus_cli` | Install `tyrus` CLI globally |
| `cargo nextest run --workspace` | Run all tests (parallel, preferred) |
| `cargo test --workspace` | Run all tests (legacy runner) |
| `cargo test -p integration_tests` | Integration tests only |
| `cargo clippy --workspace` | Lint with strict rules (`-Dwarnings` enforced) |
| `cargo fmt -- --check` | Check formatting |
| `cargo insta review` | Review snapshot changes |

**Tyrus CLI Commands (after global install):**

| Command | Description |
|---------|-------------|
| `tyrus check <file.ts>` | Analyze a TypeScript file for Oxidizable compatibility |
| `tyrus check --json <file.ts>` | JSON diagnostic output (for tooling integration) |
| `tyrus build <file.ts>` | Transpile to Rust (stdout) |
| `tyrus build <file.ts> -o output.rs` | Transpile to Rust (file) |
| `tyrus build <dir> -o <output_dir>` | Transpile directory to Cargo project |
| `tyrus compile <file.ts> -o <output_dir>` | Transpile + compile to native binary |
| `tyrus compile <file.ts> --release` | Transpile + compile with optimizations |
| `tyrus run <file.ts>` | Transpile + compile + execute |
| `tyrus --quiet <command>` | Suppress banner for scripting |
<!-- /AUTO-GENERATED -->

---

## 🧪 Test Suite

158 tests across 7 test types and 4 feature tiers:

| Type | Count | Description |
|------|-------|-------------|
| **Equivalence** | 55 | Semantic proof: TS and Rust produce identical stdout |
| **CLI** | 7 | Integration tests for all CLI commands and flags |
| **Unit** | 27 | Fast, isolated codegen function tests |
| **Snapshot** | 6 | Full transpilation output via `insta` |
| **Compilation** | 54 | Generated Rust passes `cargo check` per tier |
| **IR** | 8 | Typed intermediate representation lowering |
| **Trybuild** | 1 | Compile-verification of generated Rust |

Test types: **Equivalence** (TS↔Rust same output) · **CLI** (command integration) · **Unit** (fast, isolated functions) · **Snapshot** (insta, codegen output) · **Compilation** (generated Rust passes `cargo check`) · **IR** (type lowering) · **Trybuild** (compile-verification)

---

## 📖 Thesis & Architecture

For a deep dive into the compiler's internals, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## 🏗 Code Generation Module Structure

The `tyrus_codegen` crate is organized into focused, single-responsibility modules under `crates/tyrus_codegen/src/convert/`:

```
convert/
├── mod.rs          — module declarations and re-exports
├── interface.rs    — RustGenerator struct definition + Visit impl (entry point)
├── helpers.rs      — shared utilities: to_snake_case, to_pascal_case, is_string_expr
├── stmt.rs         — statement conversion (convert_stmt, convert_stmt_recursive)
├── fn_decl.rs      — function declaration processing (process_fn_decl)
├── module.rs       — module/import handling
├── type_mapper.rs  — TypeScript → Rust type mapping (deduplicated map_type_core)
├── class/          — class → struct+impl (split from monolithic class.rs)
│   ├── mod.rs          — dispatcher + property conversion
│   ├── constructor.rs  — constructor transpilation + DI
│   ├── method.rs       — method transpilation + decorators
│   ├── routing.rs      — Axum router generation + FromRequestParts
│   └── mutation.rs     — self-mutation detection
└── expr/
    ├── mod.rs      — expression dispatcher (convert_expr)
    ├── binary.rs   — binary operators (convert_bin_expr)
    ├── call.rs     — function/method calls, axios/fetch/array methods
    ├── member.rs   — property access, mutex state (convert_member_expr)
    ├── arrow.rs    — arrow functions → closures (convert_arrow_expr)
    ├── literal.rs  — literals, object/array/template expressions
    └── misc.rs     — assignments, updates, optional chaining
```

All Rust code is generated using `quote!` macros producing `proc_macro2::TokenStream` — never string concatenation.

## 📄 License

MIT License. See [LICENSE](LICENSE) for details.
