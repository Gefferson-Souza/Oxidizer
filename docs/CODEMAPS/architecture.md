<!-- Generated: 2026-03-15 | Files scanned: 62 | Token estimate: ~800 -->

# Tyrus Architecture Codemap

## Pipeline Flow

```
.ts input
  → tyrus_parser    (SWC → swc_ecma_ast::Program)
  → tyrus_analyzer  (LintVisitor + DecoratorVisitor → DiGraph)
  → tyrus_codegen   (RustGenerator → proc_macro2::TokenStream)
  → tyrus_orchestrator (pipeline + scaffold + format → .rs output)
```

## Crate Dependency Graph

```
tyrus_cli ─────→ tyrus_orchestrator ──→ tyrus_codegen
                       │                      │
                       ├──→ tyrus_parser       ├──→ quote/proc_macro2
                       ├──→ tyrus_analyzer     │
                       ├──→ tyrus_di           │
                       └──→ tyrus_diagnostics  │
                                               │
                 tyrus_ast (IR, standalone)     │
                 tyrus_common (shared utils)    │
                 tyrus_test_utils (test helpers)│
```

## Entry Points

| Binary | Crate | Entry |
|--------|-------|-------|
| `tyrus` CLI | tyrus_cli | `crates/tyrus_cli/src/main.rs` |
| Test suite | integration_tests | `tests/src/lib.rs` + `tests/tests/tier4_tests.rs` |
| Benchmarks | tyrus_orchestrator | `crates/tyrus_orchestrator/benches/` |

## Data Flow: Single File

```
tyrus check file.ts
  1. parse(file.ts) → Program (SWC AST)
  2. Analyzer::analyze(program) → AnalysisResult { errors, diagnostics, graph }
  3. Report errors/diagnostics

tyrus build file.ts
  1-2. Same as check
  3. codegen::generate(program, is_index) → GeneratedCode { code, controllers }
  4. format_code(code) → prettyplease formatted Rust
  5. Output to stdout or file

tyrus compile dir/src/ -o dir/output/
  1. Walk dir → parse all .ts files → Vec<Program>
  2. Analyze each → merge DiGraph
  3. graph.resolve() → topological init order
  4. Generate each file → write .rs
  5. generate_mod_rs() per directory
  6. generate_main_rs(init_order, controllers)
  7. generate_cargo_toml()
  8. cargo build
```

## Key Structs

| Struct | Crate | Role |
|--------|-------|------|
| `RustGenerator` | tyrus_codegen | Main visitor, holds all codegen state |
| `DiGraph` | tyrus_di | petgraph-based DI resolution |
| `Module` | tyrus_di | NestJS module metadata |
| `TyrusError` | tyrus_diagnostics | miette-based error type |
| `ControllerMetadata` | tyrus_codegen | Tracks controller names + routes |

## Stats

| Metric | Value |
|--------|-------|
| Total Rust lines | 9,044 |
| Crates | 10 |
| Codegen modules | 32 files |
| Tests | 195 (81 equivalence) |
