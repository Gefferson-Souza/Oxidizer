# Transpiler Benchmarks

This directory documents the performance benchmarking infrastructure for the Tyrus transpiler.

## Status

Implemented. Criterion.rs benchmarks measure transpiler throughput (TypeScript to Rust conversion speed). Located in `crates/tyrus_orchestrator/benches/transpiler.rs`.

## Methodology

We use [Criterion.rs](https://github.com/bheisler/criterion.rs) for statistically significant benchmarking with automatic warm-up, outlier detection, and HTML report generation.

### Metrics

1. **Execution Time:** Wall-clock time per transpilation invocation.
2. **Throughput:** Iterations per second for each scenario.
3. **Statistical Confidence:** Criterion computes confidence intervals and detects regressions automatically.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench -p tyrus_orchestrator

# Run a specific benchmark by name
cargo bench -p tyrus_orchestrator -- transpile_simple_functions

# Compile without running (CI validation)
cargo bench -p tyrus_orchestrator --no-run
```

HTML reports are generated in `target/criterion/` after each run.

## Scenarios

| Benchmark | Description | Source Pattern |
|-----------|-------------|----------------|
| `transpile_simple_functions` | Three typed functions (arithmetic, boolean, template literal) | Tier 1 functions |
| `transpile_interfaces` | Three interfaces including optional fields and nested types | Tier 2 interfaces |
| `transpile_class_with_methods` | Class with private field, constructor, methods, and self-mutation | Tier 2 classes |
| `transpile_combined_project` | Mixed file with functions, control flow, interface, and class | Multi-construct file |

Each scenario writes TypeScript source to a temporary file and benchmarks the full `build()` pipeline: parsing, code generation, and formatting.
