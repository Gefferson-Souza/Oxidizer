# Tyrus Benchmark Suite

Performance validation for the Tyrus TypeScript-to-Rust transpiler.

## Quick Start

```bash
# Run transpilation speed benchmarks (criterion.rs)
cargo bench --bench transpiler

# Run runtime comparison (Node.js vs compiled Rust)
cargo bench --bench runtime_comparison

# Run with CSV output (for thesis data)
BENCH_CSV=1 cargo bench --bench runtime_comparison

# Quick mode (fewer iterations)
cargo bench --bench transpiler -- --quick

# Run a specific benchmark group
cargo bench --bench transpiler -- full_pipeline
cargo bench --bench transpiler -- pipeline_stages
cargo bench --bench transpiler -- scalability
```

## Benchmark Types

### 1. Transpilation Speed (`transpiler`)

Measures how fast Tyrus converts TypeScript to Rust using [criterion.rs](https://github.com/bheisler/criterion.rs).

**Three benchmark groups:**

| Group | What it measures | Inputs |
|-------|-----------------|--------|
| `full_pipeline` | End-to-end transpilation by complexity tier | Tier 1-4 fixtures + combined |
| `pipeline_stages` | Individual stage timing (parse, analyze, codegen, format) | Combined input |
| `scalability` | Linear scaling with code size | 1, 5, 10, 20 functions |

**Example output:**

```
full_pipeline/transpile/tier1_functions        time: [47.4 µs  47.7 µs  48.9 µs]
full_pipeline/transpile/tier2_class            time: [103  µs  106  µs  107  µs]
full_pipeline/transpile/tier3_stdlib           time: [144  µs  148  µs  149  µs]
full_pipeline/transpile/tier4_nestjs           time: [676  µs  677  µs  677  µs]

pipeline_stages/1_parse                        time: [19.1 µs  19.1 µs  19.3 µs]
pipeline_stages/2_parse_analyze                time: [22.2 µs  22.3 µs  22.7 µs]
pipeline_stages/3_parse_codegen                time: [87.7 µs  88.5 µs  88.7 µs]
pipeline_stages/4_full_pipeline                time: [315  µs  316  µs  318  µs]
```

**Key findings:**
- Parsing is fast (~19µs) — SWC is highly optimized
- Codegen dominates (~88µs for combined input)
- Formatting (prettyplease) adds ~230µs
- NestJS tier (decorators, DI) is ~14x slower than basic functions

### 2. Runtime Comparison (`runtime_comparison`)

Measures execution speed of identical algorithms in Node.js vs compiled Rust.

**Test cases:**

| Algorithm | Description | Complexity |
|-----------|-------------|------------|
| `fibonacci` | Recursive fibonacci(30) | O(2^n) |
| `sum_loop` | Sum 1 to 1,000,000 | O(n) |
| `string_build` | String concatenation 1000x | O(n) |
| `math_intensive` | sqrt + floor in loop 10K | O(n) |
| `nested_loops` | Nested O(n^2) with n=500 | O(n^2) |

**Example output:**

```
╔══════════════════════════════════════════════════════════════════════════╗
║              Tyrus Runtime Comparison: Node.js vs Rust                  ║
╠══════════════════════════════════════════════════════════════════════════╣
║ Benchmark            │      Node.js │         Rust │  Speedup │  Match ║
╠══════════════════════════════════════════════════════════════════════════╣
║ fibonacci            │      47.34ms │       4.79ms │    9.9x │     OK ║
║ sum_loop             │      43.43ms │       1.64ms │   26.5x │     OK ║
║ string_build         │      40.16ms │       0.96ms │   41.9x │     OK ║
║ math_intensive       │      42.59ms │       0.77ms │   55.2x │     OK ║
║ nested_loops         │      42.12ms │       0.99ms │   42.4x │     OK ║
╠══════════════════════════════════════════════════════════════════════════╣
║ AVERAGE              │              │              │   35.2x │ ALL OK ║
╚══════════════════════════════════════════════════════════════════════════╝

  Semantic equivalence: PROVEN
```

**Methodology:**
1. Same TypeScript code runs in both Node.js and Tyrus-compiled Rust
2. Output is compared line-by-line for semantic equivalence
3. Each benchmark runs N iterations (configurable via `BENCH_ITERATIONS`)
4. Rust binaries are compiled with `--release` optimizations
5. Node.js uses `--experimental-strip-types` (Node 22+)

## CI Integration

Benchmarks run automatically in GitHub Actions after tests pass. Criterion HTML reports are uploaded as artifacts (retained 30 days).

```yaml
# In .github/workflows/ci.yml
bench:
  name: Benchmarks
  needs: build
  steps:
    - cargo bench --bench transpiler
    - BENCH_ITERATIONS=3 cargo bench --bench runtime_comparison
    - Upload criterion reports as artifacts
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `BENCH_ITERATIONS` | `5` | Number of timed runs per benchmark (runtime comparison) |
| `BENCH_CSV` | unset | When set, outputs machine-readable CSV after the table |

## Reproducing Results

For thesis-quality reproducible results:

```bash
# Ensure clean build
cargo clean

# Run with sufficient iterations
BENCH_ITERATIONS=10 BENCH_CSV=1 cargo bench --bench runtime_comparison 2>/dev/null

# Full criterion benchmarks with HTML report
cargo bench --bench transpiler
open target/criterion/report/index.html
```

## File Locations

| File | Purpose |
|------|---------|
| `crates/tyrus_orchestrator/benches/transpiler.rs` | Criterion transpilation benchmarks |
| `crates/tyrus_orchestrator/benches/runtime_comparison.rs` | Node.js vs Rust comparison |
| `target/criterion/` | Generated HTML reports |
