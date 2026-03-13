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

- **String Methods** (14): `includes`, `replace`, `split`, `toUpperCase`, `toLowerCase`, `trim`, `startsWith`, `endsWith`, `toString`, `substring`, `charAt`, `indexOf`, `repeat`, `slice`
- **Array Methods** (14): `map`, `filter`, `forEach`, `find`, `some`, `every`, `reduce`/`fold`, `join`, `includes`, `push`, `indexOf`, `slice`, `concat`, `reverse`, `pop`
- **Math Functions** (16): `max`, `min`, `round`, `floor`, `ceil`, `abs`, `random`, `pow`, `sqrt`, `log`, `trunc`, `sign`, `sin`, `cos`, `tan`, spread variants
- **Math Constants**: `Math.PI`, `Math.E`
- **Console**: `log`, `error`, `warn`, `info`, `debug`
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

### Using the Compiler

```bash
# Analyze a TypeScript file for compatibility
./target/release/tyrus check ./src/index.ts

# Get JSON diagnostics (for tooling integration)
./target/release/tyrus check --json ./src/index.ts

# Transpile to Rust source code
./target/release/tyrus build ./src/index.ts

# Transpile + compile to native binary
./target/release/tyrus compile ./src/ --output ./output

# Transpile + compile + execute
./target/release/tyrus run ./src/ --output ./output

# Suppress banner for scripting
./target/release/tyrus --quiet check ./src/index.ts
```

---

## 📋 Commands Reference

<!-- AUTO-GENERATED from Cargo.toml and CI -->
| Command | Description |
|---------|-------------|
| `cargo build --workspace` | Build all workspace crates |
| `cargo build --release` | Production build with LTO |
| `cargo nextest run --workspace` | Run all tests (parallel, preferred) |
| `cargo test --workspace` | Run all tests (legacy runner) |
| `cargo test -p integration_tests` | Integration tests only |
| `cargo clippy --workspace` | Lint with strict rules (`-Dwarnings` enforced) |
| `cargo fmt -- --check` | Check formatting |
| `cargo insta review` | Review snapshot changes |
| `cargo run --bin tyrus -- check <file.ts>` | Analyze a TypeScript file for compatibility |
| `cargo run --bin tyrus -- check --json <file.ts>` | JSON diagnostic output (for tooling) |
| `cargo run --bin tyrus -- build <dir>/src --output <dir>/output` | Transpile to a complete Rust project |
| `cargo run --bin tyrus -- compile <dir>/src --output <dir>/output` | Transpile + compile to native binary |
| `cargo run --bin tyrus -- run <dir>/src --output <dir>/output` | Transpile + compile + execute |
<!-- /AUTO-GENERATED -->

---

## 🧪 Test Suite

146 tests across 5 test types and 4 feature tiers:

| Type | Count | Description |
|------|-------|-------------|
| **Equivalence** | 51 | Semantic proof: TS and Rust produce identical stdout |
| **Unit** | 27 | Fast, isolated codegen function tests |
| **Snapshot** | 6 | Full transpilation output via `insta` |
| **Compilation** | 54 | Generated Rust passes `cargo check` per tier |
| **IR** | 8 | Typed intermediate representation lowering |

Test types: **Equivalence** (TS↔Rust same output) · **Unit** (fast, isolated functions) · **Snapshot** (insta, codegen output) · **Compilation** (generated Rust passes `cargo check`) · **IR** (type lowering)

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
