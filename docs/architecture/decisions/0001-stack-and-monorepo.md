# 1. Technology Stack and Monorepo Structure

Date: 2025-11-22
Status: Accepted

## Context
The goal is to build **Tyrus**, a tool that analyzes TypeScript code and transpiles it into idiomatic Rust.
The main requirements are:
1. **Performance:** The tool must process large projects quickly.
2. **Reliability:** The generated code must be safe, and the tool must not fail unexpectedly.
3. **Maintainability:** The project must be modular to allow isolated testing of the parser, analyzer, and codegen.
4. **Ecosystem:** We need robust tooling for JS/TS parsing, since writing a parser from scratch is infeasible for this scope.

## Decision
We decided to use the **Rust** language to build the compiler, organized as a **Cargo Workspace (Monorepo)**.

### Selected Stack:
- **Language:** Rust (memory safety, type system, performance).
- **Parsing:** `swc_ecma_parser` (industry-standard library, written in Rust, extremely fast).
- **AST Traversal:** `swc_ecma_visit` (Visitor pattern implementation for efficient tree navigation).
- **Code Generation:** `quote!` and `proc_macro2` (hygienic Rust token generation).
- **CLI:** `clap` v4 (standard for command-line interfaces).
- **Error Reporting:** `miette` (rich diagnostics with visual source code support).
- **Testing:** `insta` (snapshot testing) and `trybuild` (compilation testing).

### Module (Crate) Structure:
The project is split into isolated crates to ensure separation of concerns (SoC):
- `tyrus_cli`: Entry point interface (clap).
- `tyrus_parser`: Wrapper over SWC (`swc_ecma_parser`).
- `tyrus_analyzer`: Semantic validation (`LintVisitor` + `DecoratorVisitor`).
- `tyrus_codegen`: AST-to-Rust-tokens transformation (`quote!`).
- `tyrus_orchestrator`: Multi-file pipeline orchestration.
- `tyrus_di`: Dependency Injection engine (petgraph).
- `tyrus_diagnostics`: Centralized errors (`TyrusError` + miette).
- `tyrus_common`: Shared types (`FilePath`, utilities).
- `tyrus_ast`: Reserved for a future IR (SWC AST used directly for now).
- `tyrus_test_utils`: Test helpers (`assert_rust_compiles()`).

## Consequences

### Positive
- **Native Performance:** Using Rust and SWC guarantees speed superior to tools written in JS/TS.
- **Modularity:** The Monorepo allows building and testing the *Analyzer* without needing to run the *Codegen*.
- **Strong Typing:** Rust's type system prevents internal logic errors in the compiler.
- **Academic Rigor:** The pipeline architecture (Parse -> Analyze -> Generate) is classic in compiler theory.

### Negative
- **Learning Curve:** The SWC API is complex and poorly documented.
- **Compile Time:** Rust has slow compile times, which can affect the feedback cycle (mitigated by using separate crates).
