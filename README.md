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

### 📦 Supported Patterns (Verified)

- **Array Literals**: `[1, 2, 3]` -> `vec![1, 2, 3]`
- **Computed Properties**: `obj["key"]` -> `obj["key"]` (via serde_json)
- **Class State**: Automatic `Arc<Mutex<T>>` wrapping for services/controllers.
- **DTOs**: Pure structs for data transfer objects.
- **Standard Lib**: `map`, `filter`, `find`, `push` mapped to Rust equivalents.
- **String Replace**: `str.replace(a, b)` -> `str.replacen(a, b, 1)` (Exact JS semantics).

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

### Compiling a Project

```bash
# Analyze a TypeScript file for compatibility
./target/release/tyrus check ./src/index.ts

# Transpile to a complete Rust project
./target/release/tyrus build ./src/index.ts
```

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
├── class.rs        — class → struct+impl conversion, Arc<Mutex<T>> state management
├── module.rs       — module/import handling
├── type_mapper.rs  — TypeScript → Rust type mapping
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
