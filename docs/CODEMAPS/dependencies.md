<!-- Generated: 2026-03-15 | Files scanned: 10 | Token estimate: ~400 -->

# Dependencies Codemap

## Compiler Dependencies (Cargo workspace)

| Crate | Version | Purpose |
|-------|---------|---------|
| swc_ecma_parser | 0.143 | TypeScript parsing |
| swc_ecma_ast | 0.110 | AST types |
| swc_ecma_visit | 0.96 | Visitor pattern |
| swc_common | 0.33 | Source maps, spans |
| quote | 1.0 | Rust code generation |
| proc_macro2 | 1.0 | Token streams |
| syn | 2.0 | Rust AST parsing (prettyplease) |
| prettyplease | 0.2 | Rust code formatting |
| petgraph | 0.6 | DI dependency graph |
| miette | 7.0 | Error reporting |
| thiserror | 2.0 | Error derives |
| clap | 4.0 | CLI argument parsing |
| walkdir | 2.0 | Directory traversal |
| serde/serde_json | 1.0 | Serialization |

## Generated Project Dependencies (Cargo.toml output)

| Crate | Purpose |
|-------|---------|
| tokio | Async runtime |
| axum 0.7 | HTTP framework |
| serde + serde_json | JSON serialization |
| reqwest 0.12 | HTTP client (fetch/axios) |
| tower + tower-http | Middleware layers |
| rand 0.8 | Math.random() |

## Test Dependencies

| Crate | Purpose |
|-------|---------|
| insta | Snapshot testing |
| trybuild | Compile-verification |
| tempfile | Temp files for equivalence tests |
| criterion | Benchmarking |

## External Tools

| Tool | Purpose |
|------|---------|
| Node.js 22+ | Run TS for equivalence tests |
| cargo nextest | Parallel test runner |
| cargo clippy | Lint (strict rules) |
| hey / bombardier | HTTP load testing |
