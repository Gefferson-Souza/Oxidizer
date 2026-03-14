# Academic Benchmark Suite: Real-World Performance — Node.js vs Tyrus-Compiled Rust

## Objective

Measure and compare the performance of **real-world computational patterns** commonly found in production TypeScript/Node.js applications, transpiled by Tyrus to native Rust binaries. The benchmarks simulate actual bottlenecks identified in production Node.js systems.

## Research Context

### Why These Benchmarks Matter

Node.js excels at I/O-bound workloads but struggles with CPU-intensive operations due to its single-threaded event loop model. According to industry benchmarks (TechEmpower 2025), Rust achieves **5-50x better performance** than Node.js in CPU-bound scenarios. The key question this benchmark answers:

> **Can an academic transpiler (Tyrus) produce Rust code that captures these performance gains automatically, while preserving semantic equivalence?**

### Identified Real-World Bottlenecks (from research)

| Bottleneck | Where It Happens | Why It's Slow |
|------------|-----------------|---------------|
| Data transformation pipelines | API response processing, ETL jobs | map/filter/reduce chains create intermediate arrays, GC pressure |
| Sorting large datasets | Database result ordering, leaderboards | Array.sort() is CPU-bound, blocks event loop |
| String processing | Log parsing, text search, CSV generation | Immutable strings → O(n²) concatenation, heavy allocation |
| Statistical computation | Analytics dashboards, ML preprocessing | Math operations in tight loops block event loop |
| Nested iteration | Report generation, matrix operations | O(n²) loops saturate single thread |
| Array accumulation | Building responses, aggregating data | Push + GC pressure on large arrays |

Sources:
- [V8's JSON.stringify Speed Boost (2025)](https://biggo.com/news/202508050113_V8_JSON_stringify_performance_boost)
- [CPU-Intensive Tasks Kill API Performance](https://dev-aditya.medium.com/the-dark-side-of-node-js-e8e38bd171c0)
- [Node.js Performance Best Practices 2025](https://dev.to/satyam_gupta_0d1ff2152dcc/boost-your-apps-top-nodejs-performance-best-practices-for-2025-3cco)
- [Rust vs Node.js Performance 2025](https://dev.to/hamzakhan/rust-vs-nodejs-vs-go-performance-comparison-for-backend-development-2g69)

---

## Task Type
- [x] Backend (data processing, computation, performance measurement)

## Methodology

### Academic Rigor
1. **Reproducibility** — Docker containers with pinned versions (Node 22.x, Rust stable)
2. **Statistical validity** — 13 runs per test (3 warmup + 10 measured), report mean ± std dev
3. **Isolation** — Each benchmark in its own process, no shared state
4. **Equivalence proof** — Both programs must produce IDENTICAL output (stdout comparison)
5. **Full documentation** — Every CLI step logged with exact commands
6. **Comprehensive metrics** — Time, CPU, memory, binary size

### What We Measure

| Metric | How | Unit |
|--------|-----|------|
| Wall-clock execution time | `time` command | milliseconds |
| Peak memory (RSS) | `/usr/bin/time -v` → "Maximum resident set size" | KB |
| CPU usage | `/usr/bin/time -v` → "Percent of CPU" | % |
| Binary size | `ls -la` (Rust binary vs Node.js runtime) | bytes |
| Transpilation time | `time tyrus build` | milliseconds |
| Lines of generated code | `wc -l` | lines |

---

## Benchmark Programs (6 Real-World Scenarios)

### Test 1: Data Pipeline — ETL-style filter/map/reduce on 100K records

**Real-world analog:** API backend processing database results, filtering invalid entries, transforming fields, computing aggregates. This is the #1 pattern in Express/NestJS controllers.

```typescript
function dataPipeline(size: number): number {
    const data: number[] = [];
    let i: number = 0;
    while (i < size) {
        data.push(i * 1.0);
        i = i + 1;
    }

    const result: number = data
        .filter((n: number) => n % 3 !== 0)
        .map((n: number) => n * n + Math.sqrt(n))
        .reduce((acc: number, n: number) => acc + n, 0);

    return Math.floor(result);
}

function main(): void {
    console.log(dataPipeline(100000));
}
main();
```

**What it stresses:** Array allocation, iterator chains, math in closures, GC pressure from intermediate arrays.

---

### Test 2: Statistical Analysis — Mean, Variance, Std Dev on 1M data points

**Real-world analog:** Analytics dashboards computing statistics over user data, ML feature engineering, monitoring systems computing percentiles.

```typescript
function statistics(size: number): string {
    const data: number[] = [];
    let i: number = 0;
    while (i < size) {
        data.push(Math.sin(i * 0.001) * 100 + Math.cos(i * 0.002) * 50);
        i = i + 1;
    }

    let sum: number = 0;
    let j: number = 0;
    while (j < size) {
        sum = sum + data[j];
        j = j + 1;
    }
    const mean: number = sum / size;

    let variance_sum: number = 0;
    let k: number = 0;
    while (k < size) {
        const diff: number = data[k] - mean;
        variance_sum = variance_sum + diff * diff;
        k = k + 1;
    }
    const variance: number = variance_sum / size;
    const std_dev: number = Math.sqrt(variance);

    return Math.floor(mean).toString() + "," + Math.floor(std_dev).toString();
}

function main(): void {
    console.log(statistics(1000000));
}
main();
```

**What it stresses:** Large array allocation (1M f64), multiple passes over data, Math operations, arithmetic in tight loops.

---

### Test 3: Text Processing — CSV-like parsing and transformation

**Real-world analog:** Log parsing, CSV export generation, template rendering, text search in large strings. String operations are inherently expensive in JS due to immutable strings.

```typescript
function textProcessing(lines: number): string {
    let csv: string = "";
    let i: number = 0;
    while (i < lines) {
        const name: string = "user_" + i.toString();
        const score: string = Math.floor(Math.random() * 100).toString();
        csv = csv + name + "," + score + "\n";
        i = i + 1;
    }

    let count: number = 0;
    const rows: string[] = csv.split("\n");
    rows.forEach((row: string) => {
        if (row.includes("user_1")) {
            count = count + 1;
        }
    });

    return count.toString();
}

function main(): void {
    console.log(textProcessing(10000));
}
main();
```

**What it stresses:** String concatenation (O(n²) in naive JS), split(), includes(), forEach on string arrays. This is where JS's immutable strings really hurt.

---

### Test 4: Sorting — Sorting 100K numbers (leaderboard simulation)

**Real-world analog:** Sorting database results client-side, leaderboard computation, ranking algorithms, priority queue processing.

```typescript
function sortBenchmark(size: number): string {
    const data: number[] = [];
    let i: number = 0;
    while (i < size) {
        data.push(Math.floor(Math.sin(i * 1.0) * 1000000));
        i = i + 1;
    }

    data.sort();

    return data[0].toString() + "," + data[size - 1].toString();
}

function main(): void {
    console.log(sortBenchmark(100000));
}
main();
```

**What it stresses:** Array.sort() with comparison function, cache locality, branch prediction. Sort is one of the most CPU-intensive operations in real APIs.

---

### Test 5: Nested Computation — Matrix-like O(n²) operation

**Real-world analog:** Report generation with cross-references, permission matrix computation, distance calculations, recommendation engine scoring.

```typescript
function matrixCompute(n: number): number {
    let total: number = 0;
    let i: number = 0;
    while (i < n) {
        let j: number = 0;
        while (j < n) {
            total = total + Math.floor(Math.sqrt(i * j + 1));
            j = j + 1;
        }
        i = i + 1;
    }
    return total;
}

function main(): void {
    console.log(matrixCompute(3000));
}
main();
```

**What it stresses:** O(n²) = 9M iterations, Math.sqrt + Math.floor in inner loop, integer arithmetic. This simulates any quadratic algorithm that blocks the Node.js event loop.

---

### Test 6: Data Accumulation — Building large structured results

**Real-world analog:** Building API responses by accumulating data from multiple sources, constructing reports, aggregating metrics. Tests GC pressure from large array growth.

```typescript
function accumulate(iterations: number): string {
    const results: number[] = [];
    let i: number = 0;
    while (i < iterations) {
        const value: number = Math.floor(Math.pow(i * 1.0, 1.5)) + Math.abs(Math.sin(i * 1.0) * 100);
        results.push(value);
        i = i + 1;
    }

    let sum: number = 0;
    results.forEach((v: number) => {
        sum = sum + v;
    });

    const avg: number = Math.floor(sum / iterations);
    return results.length.toString() + "," + avg.toString();
}

function main(): void {
    console.log(accumulate(500000));
}
main();
```

**What it stresses:** Array.push() growth (reallocation), Math operations, forEach iteration, GC pressure from 500K elements.

---

## Tyrus Capability Verification

Before running benchmarks, each program MUST be verified:

| Feature Used | Tyrus Support | Status |
|-------------|---------------|--------|
| `while` loops | Phase 4 | ✅ |
| `Math.floor/sqrt/sin/cos/abs/pow/random` | Phase 5 | ✅ |
| `Array.push/filter/map/reduce/forEach/sort` | Phase 5 | ✅ |
| `String.split/includes/toString` | Phase 5 | ✅ |
| String concatenation (`+`) | Phase 1 | ✅ |
| Top-level statements (`function main()`) | Phase 6.2 | ✅ |
| Array spread (`[...arr]`) | Phase 6.3 | ✅ |
| Arrow functions in callbacks | Phase 1 | ✅ |
| Template literals | Phase 1 | ✅ |
| Array indexing (`data[i]`) | Phase 1 | ⚠️ (f64 index needs cast) |

**Known limitation:** `data[j]` where `j` is `f64` needs `as usize` cast. Tests 2 and 4 use array indexing — these may need the `while` loop counter to be used differently, or the transpiler may handle it. We verify before running.

---

## Implementation Steps

### Step 1: Create benchmark infrastructure
```
benchmarks/academic/
├── programs/                    # 6 TypeScript source files
│   ├── test1_data_pipeline.ts
│   ├── test2_statistics.ts
│   ├── test3_text_processing.ts
│   ├── test4_sorting.ts
│   ├── test5_matrix_compute.ts
│   └── test6_accumulation.ts
├── docker/
│   ├── Dockerfile.node          # Node.js 22 container
│   └── Dockerfile.rust          # Multi-stage: tyrus compile + run
├── scripts/
│   ├── run_all.sh               # Main orchestrator (documented steps)
│   ├── measure.sh               # Single program measurement
│   ├── verify_equivalence.sh    # Compare Node vs Rust outputs
│   └── generate_report.sh       # Statistics + formatted output
├── results/                     # Raw CSV data + reports
│   ├── raw/                     # Per-run timing data
│   └── report.md                # Final academic report
├── tyrus_output/                # Generated Rust code (for inspection)
└── README.md                    # Full methodology documentation
```

### Step 2: Write 6 TypeScript programs
Each file is standalone, uses only supported Tyrus features.

### Step 3: Document Tyrus CLI pipeline (for each program)

```bash
# 3a. Analyze — show the analyzer in action
echo "=== STEP 1: tyrus check ==="
tyrus check programs/test1_data_pipeline.ts
echo "Result: Compatible with Oxidizable Standard"

# 3b. Check with JSON output (machine-readable diagnostics)
echo "=== STEP 2: tyrus check --json ==="
tyrus check --json programs/test1_data_pipeline.ts

# 3c. Transpile — show generated Rust code
echo "=== STEP 3: tyrus build ==="
tyrus build programs/test1_data_pipeline.ts > tyrus_output/test1.rs
echo "Generated $(wc -l < tyrus_output/test1.rs) lines of Rust"

# 3d. Compile — build native binary
echo "=== STEP 4: tyrus compile ==="
time tyrus compile programs/test1_data_pipeline.ts --output build/test1

# 3e. Run — execute and capture output
echo "=== STEP 5: tyrus run ==="
tyrus run programs/test1_data_pipeline.ts --output build/test1

# 3f. Compare with Node.js
echo "=== STEP 6: Equivalence check ==="
node --experimental-strip-types programs/test1_data_pipeline.ts > /tmp/node_out.txt
./build/test1/target/release/tyrus_app > /tmp/rust_out.txt
diff /tmp/node_out.txt /tmp/rust_out.txt && echo "EQUIVALENT ✅"
```

### Step 4: Create Docker containers

**Dockerfile.node:**
```dockerfile
FROM node:22-slim
WORKDIR /bench
COPY programs/ ./programs/
# Measure inside container to avoid Docker overhead
ENTRYPOINT ["node", "--experimental-strip-types"]
```

**Dockerfile.rust:**
```dockerfile
# Stage 1: Build with Tyrus
FROM rust:1.75-slim AS builder
# Install tyrus from source
COPY . /tyrus
WORKDIR /tyrus
RUN cargo build --release --bin tyrus
# Transpile and compile each program
COPY programs/ /programs/
RUN for f in /programs/*.ts; do \
      /tyrus/target/release/tyrus compile "$f" --output "/build/$(basename $f .ts)"; \
    done

# Stage 2: Run
FROM debian:bookworm-slim
COPY --from=builder /build/ /bench/
ENTRYPOINT ["/bench/test1/target/release/tyrus_app"]
```

### Step 5: Measurement script
```bash
#!/bin/bash
# measure.sh — Run a single benchmark with full metrics
PROGRAM=$1
RUNTIME=$2  # "node" or "rust"
ITERATIONS=${3:-13}
WARMUP=3

for i in $(seq 1 $ITERATIONS); do
    if [ "$RUNTIME" = "node" ]; then
        /usr/bin/time -v node --experimental-strip-types "programs/$PROGRAM.ts" \
            2>&1 | grep -E "wall clock|Maximum resident|CPU"
    else
        /usr/bin/time -v "./build/$PROGRAM/target/release/tyrus_app" \
            2>&1 | grep -E "wall clock|Maximum resident|CPU"
    fi
done
```

### Step 6: Generate academic report
Parse CSV timing data, compute:
- Mean execution time ± standard deviation
- Peak memory usage (RSS)
- CPU utilization percentage
- Speedup ratio (Node time / Rust time)
- Memory ratio (Node RSS / Rust RSS)

## Expected Output Format

```
╔══════════════════════════════════════════════════════════════════════════════════════╗
║        Academic Benchmark: Node.js 22 vs Tyrus-Compiled Rust (Release)              ║
║        Environment: Docker, 10 measured runs, 3 warmup discarded                    ║
╠══════════════════════════════════════════════════════════════════════════════════════╣
║ Scenario           │ Node.js (ms)  │ Rust (ms)   │ Speedup │ Memory  │ Equivalent ║
╠══════════════════════════════════════════════════════════════════════════════════════╣
║ Data Pipeline 100K │ xxx ± yy      │ xxx ± yy    │ XX.Xx   │ X.Xx    │ ✅         ║
║ Statistics 1M      │ xxx ± yy      │ xxx ± yy    │ XX.Xx   │ X.Xx    │ ✅         ║
║ Text Processing    │ xxx ± yy      │ xxx ± yy    │ XX.Xx   │ X.Xx    │ ✅         ║
║ Sort 100K          │ xxx ± yy      │ xxx ± yy    │ XX.Xx   │ X.Xx    │ ✅         ║
║ Matrix O(n²)       │ xxx ± yy      │ xxx ± yy    │ XX.Xx   │ X.Xx    │ ✅         ║
║ Accumulation 500K  │ xxx ± yy      │ xxx ± yy    │ XX.Xx   │ X.Xx    │ ✅         ║
╠══════════════════════════════════════════════════════════════════════════════════════╣
║ AVERAGE            │               │             │ XX.Xx   │ X.Xx    │ ALL ✅     ║
╚══════════════════════════════════════════════════════════════════════════════════════╝

Transpilation Statistics:
  Average transpilation time: XXms per file
  Average generated code: XX lines per file
  Rust binary size (release): XX KB average

Semantic Equivalence: PROVEN — all 6 programs produce identical output
```

## Key Files

| File | Operation | Description |
|------|-----------|-------------|
| `benchmarks/academic/programs/*.ts` | Create | 6 TypeScript benchmark programs |
| `benchmarks/academic/docker/Dockerfile.node` | Create | Node.js 22 runner |
| `benchmarks/academic/docker/Dockerfile.rust` | Create | Tyrus compile + Rust runner |
| `benchmarks/academic/scripts/run_all.sh` | Create | Orchestrator with full logging |
| `benchmarks/academic/scripts/measure.sh` | Create | Single measurement with /usr/bin/time |
| `benchmarks/academic/scripts/generate_report.sh` | Create | Statistics + formatted report |
| `benchmarks/academic/README.md` | Create | Academic methodology |

## Risks and Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Array indexing with f64 index | Tests 2, 4 may fail | Use while-loop counter pattern, verify with tyrus build first |
| Math.random() non-deterministic | Outputs won't match | Use deterministic seed (Math.sin(i)) instead |
| String concat O(n²) may be too slow in Node | Test 3 timeout | Cap at 10K lines |
| Sort comparison semantics differ | f64 sort in Rust differs from JS | Use default sort(), verify output matches |
| Docker startup time masks results | Inaccurate timing | Measure INSIDE container, not container startup |

## Feasibility Assessment

Based on analysis of current Tyrus capabilities (168 tests passing, 65 equivalence proven):

| Test | Feasible? | Risk |
|------|-----------|------|
| 1. Data Pipeline | ✅ Yes | filter/map/reduce all work |
| 2. Statistics | ⚠️ Likely | Array indexing (data[j]) needs f64→usize — may need adjustment |
| 3. Text Processing | ✅ Yes | split/includes/forEach all work |
| 4. Sorting | ⚠️ Likely | sort() works, array indexing for result extraction may need adjustment |
| 5. Matrix Compute | ✅ Yes | while loops + Math all work |
| 6. Accumulation | ✅ Yes | push/forEach/Math all work |

**Conclusion: 4/6 definitely feasible, 2/6 likely feasible with minor adjustments to avoid f64 array indexing.**
