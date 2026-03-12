# 🗺️ Tyrus Roadmap

This roadmap tracks the evolution of Tyrus from a research prototype to a production-grade compiler.

## ✅ Completed Milestones (Production Ready)

### 🏁 Milestone 1-3: The Foundation & Core Logic

- [x] **CLI & Parser:** Full integration with SWC for TS source analysis.
- [x] **Core Transpilation:** Arithmetic, logic, and control flow mappings.
- [x] **Semantic Analyzer:** Implementation of the "Oxidizable Standard" lints.

### 🏗️ Milestone 4-5: Type Excellence & Ecosystem

- [x] **Structural Typing:** Mapping TypeScript interfaces to Rust structs with Serde support.
- [x] **Generics:** Multi-type parameter support with trait bound inference.
- [x] **Async Revolution:** Mapping JS Promises to Rust Futures and `tokio` runtime.

### 🌐 Milestone 6: Framework Integration & Tier 3 Features

- [x] **NestJS Synthesis:** Transpiling Decorators to Axum handlers.
- [x] **Advanced Loops:** `for..in`, `do..while` mappings.
- [x] **Type Aliases:** String Unions to Enums, `Record<K,V>` to `HashMap`.
- [x] **Shim Layer:** 100% coverage of core `Math`, `String`, and `Array` methods.

### 🏛️ Milestone 7: Architecture & Dependency Injection (Tier 4)

- [x] **Dependency Injection Engine:** Custom `tyrus_di` crate for graph resolution (Services & Controllers).
- [x] **Module System:** Support for `@Module()` decorators and cross-file wiring.
- [x] **Controller Mapping:** First-class support for NestJS Controllers -> Axum Routers.

### 🛠️ Milestone 8: Generator Maturity (Production Polish)

- [x] **Unified Entry Point:** Generate `main.rs` with `tokio` runtime and `axum` server binding.
- [x] **DTO/Entity Unification:** Smart wrapping strategies to align Class (Mutex) and Interface (Raw) types.
- [x] **Thread-Safe Derives:** `PartialEq` works on DTOs (Mutex removed).

### 🔧 Milestone 9: Safe Transpilation Infrastructure

- [x] **Strict Linting:** `.cargo/config.toml` with `-Dwarnings`, enforcing `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::todo`, `clippy::unimplemented`.
- [x] **Quality Thresholds:** `clippy.toml` with cognitive complexity (15), function lines (50), and parameter count (5) limits.
- [x] **Dependency Audit:** `deny.toml` with `cargo-deny` for license and security checks.
- [x] **CI Modernization:** GitHub Actions v4, `Swatinem/rust-cache@v2`, `cargo nextest`, end-to-end demo compilation in CI.
- [x] **Panic-Free Codebase:** All `todo!()` replaced with `compile_error!()`, all `.expect()` replaced with `?` or proper error handling.
- [x] **Dead Code Removal:** `tyrus_ast` emptied (reserved for future IR), `tyrus_analyzer/graph.rs` deleted.
- [x] **Test Infrastructure Rebuild:** Structured test tiers (unit/snapshot/compilation) replacing the old ad-hoc test setup.

### 🗂️ Milestone 10: Codegen Module Decomposition

- [x] **`func.rs` Decomposed:** The monolithic 1144-line `func.rs` split into 10 focused modules:
  - `helpers.rs` — case conversion utilities and type detection helpers
  - `stmt.rs` — statement conversion logic
  - `fn_decl.rs` — function declaration processing
  - `expr/mod.rs` — expression dispatcher
  - `expr/binary.rs` — binary operator transpilation
  - `expr/call.rs` — function/method calls, array methods, axios/fetch
  - `expr/member.rs` — property access and mutex state
  - `expr/arrow.rs` — arrow functions to closures
  - `expr/literal.rs` — literals, objects, arrays, template literals
  - `expr/misc.rs` — assignments, updates, optional chaining
- [x] **`func.rs` Deleted:** All references in `interface.rs`, `module.rs`, and `class.rs` updated to import from the new modules.

---

## 🔬 Future Work (Academic Research)

### Tier 4: Advanced OOP & Metaprogramming

- [ ] **Class Inheritance:** Mapping complex prototype chains to Rust Traits and Composition.
- [ ] **Custom Decorators:** Support for user-defined metadata and proxy logic.
- [ ] **Macro System:** Compiling TypeScript template literals and type-level programming into Rust macros.

### Tier 5: Optimization & Verification

- [ ] **Formal Verification:** Mathematical proof of semantic preservation.
- [ ] **IR Optimizations:** LLVM-style passes on the Tyrus intermediate representation.
- [ ] **Cinterop:** Automated binding generation for C-compatible Rust libraries.
