# Scientific Benchmarks

This directory contains rigorous performance benchmarks to validate the efficiency of TypeRust-generated code compared to Node.js execution.

## Status

Benchmarking infrastructure is planned (Phase 2 of the Standardization Plan). The compiler itself is now production-stable with a full test suite (unit/snapshot/compilation tiers). Benchmarks of the _generated_ Rust code will follow.

## Methodology

We use [Criterion.rs](https://github.com/bheisler/criterion.rs) for statistically significant benchmarking.

### Metrics

1. **Execution Time:** Wall-clock time for specific algorithms.
2. **Throughput:** Operations per second.
3. **Memory Usage:** Peak RSS (Resident Set Size).

## Running Benchmarks

```bash
cargo bench
```

## Scenarios (Planned)

- **Fibonacci (Recursive):** Stress tests function call overhead.
- **JSON Serialization:** Stress tests struct/DTO mapping (interfaces → `#[derive(Serialize, Deserialize)]`).
- **Http Requests:** Stress tests `reqwest` vs `axios` (requires local echo server).
- **Array Operations:** Stress tests iterator chains (`.map`/`.filter`/`.find`) vs JS equivalents.
