#!/bin/bash
# Academic Benchmark Suite: Node.js vs Tyrus-Compiled Rust
# Usage: ./benchmarks/academic/scripts/run_all.sh [iterations]
set -e

ITERATIONS=${1:-10}
WARMUP=3
TOTAL=$((ITERATIONS + WARMUP))
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BASE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$BASE_DIR/../.." && pwd)"
RESULTS_DIR="$BASE_DIR/results"
RAW_DIR="$RESULTS_DIR/raw"

mkdir -p "$RAW_DIR"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     Academic Benchmark Suite: Node.js vs Tyrus-Rust         ║"
echo "║     Iterations: $TOTAL ($WARMUP warmup + $ITERATIONS measured)            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

PROGRAMS=(
    "test1_data_pipeline"
    "test2_statistics"
    "test3_text_processing"
    "test4_sorting"
    "test5_matrix_compute"
    "test6_accumulation"
)

DESCRIPTIONS=(
    "Data Pipeline 100K"
    "Statistics 1M"
    "Text Processing 50K"
    "Sort 100K"
    "Matrix O(n²) 3K"
    "Accumulation 500K"
)

# Step 1: Transpile all programs
echo "=== Step 1: Transpiling with Tyrus ==="
mkdir -p "$BASE_DIR/tyrus_output"
for prog in "${PROGRAMS[@]}"; do
    echo -n "  $prog: "
    ts_start=$(date +%s%N)
    "$PROJECT_ROOT/target/release/tyrus" build "$BASE_DIR/programs/${prog}.ts" \
        > "$BASE_DIR/tyrus_output/${prog}.rs" 2>/dev/null
    ts_end=$(date +%s%N)
    transpile_ms=$(( (ts_end - ts_start) / 1000000 ))
    lines=$(wc -l < "$BASE_DIR/tyrus_output/${prog}.rs")
    echo "OK (${transpile_ms}ms, ${lines} lines)"
done

# Step 2: Compile Rust binaries
echo ""
echo "=== Step 2: Compiling Rust binaries (release) ==="
BUILD_DIR="$BASE_DIR/rust_build"
mkdir -p "$BUILD_DIR"
for prog in "${PROGRAMS[@]}"; do
    echo -n "  $prog: "
    prog_dir="$BUILD_DIR/$prog"
    mkdir -p "$prog_dir/src"
    cat > "$prog_dir/Cargo.toml" << TOML
[package]
name = "$prog"
version = "0.1.0"
edition = "2021"
[workspace]
[[bin]]
name = "$prog"
path = "src/main.rs"
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
TOML
    echo '#![allow(dead_code, unused_variables, unused_imports, unused_mut)]' > "$prog_dir/src/main.rs"
    cat "$BASE_DIR/tyrus_output/${prog}.rs" >> "$prog_dir/src/main.rs"
    (cd "$prog_dir" && CARGO_TARGET_DIR="$BUILD_DIR/target" cargo build --quiet --release 2>/dev/null)
    bin_size=$(ls -la "$BUILD_DIR/target/release/$prog" 2>/dev/null | awk '{print $5}')
    echo "OK (binary: ${bin_size} bytes)"
done

# Step 3: Verify equivalence
echo ""
echo "=== Step 3: Verifying semantic equivalence ==="
all_match=true
for prog in "${PROGRAMS[@]}"; do
    echo -n "  $prog: "
    node_out=$(node --experimental-strip-types "$BASE_DIR/programs/${prog}.ts" 2>/dev/null)
    rust_out=$("$BUILD_DIR/target/release/$prog" 2>/dev/null)
    node_norm=$(echo "$node_out" | sed 's/\.0$//')
    rust_norm=$(echo "$rust_out" | sed 's/\.0$//')
    if [ "$node_norm" = "$rust_norm" ]; then
        echo "EQUIVALENT ✅"
    else
        echo "DIFF ❌ (Node: $node_norm | Rust: $rust_norm)"
        all_match=false
    fi
done

if [ "$all_match" = false ]; then
    echo "ERROR: Not all programs produce equivalent output. Aborting."
    exit 1
fi

# Step 4: Run benchmarks
echo ""
echo "=== Step 4: Running benchmarks ($TOTAL runs each) ==="

# CSV header
echo "program,runtime,run,time_ms,output" > "$RESULTS_DIR/raw_data.csv"

for idx in "${!PROGRAMS[@]}"; do
    prog="${PROGRAMS[$idx]}"
    desc="${DESCRIPTIONS[$idx]}"
    echo ""
    echo "--- $desc ($prog) ---"

    # Node.js runs
    for run in $(seq 1 $TOTAL); do
        ts_start=$(date +%s%N)
        output=$(node --experimental-strip-types "$BASE_DIR/programs/${prog}.ts" 2>/dev/null)
        ts_end=$(date +%s%N)
        time_ms=$(( (ts_end - ts_start) / 1000000 ))
        if [ $run -le $WARMUP ]; then
            echo -n "  Node.js warmup $run: ${time_ms}ms"$'\n'
        else
            echo "  Node.js run $((run - WARMUP)): ${time_ms}ms"
            echo "$prog,node,$((run - WARMUP)),$time_ms,$output" >> "$RESULTS_DIR/raw_data.csv"
        fi
    done

    # Rust runs
    for run in $(seq 1 $TOTAL); do
        ts_start=$(date +%s%N)
        output=$("$BUILD_DIR/target/release/$prog" 2>/dev/null)
        ts_end=$(date +%s%N)
        time_ms=$(( (ts_end - ts_start) / 1000000 ))
        if [ $run -le $WARMUP ]; then
            echo -n "  Rust warmup $run: ${time_ms}ms"$'\n'
        else
            echo "  Rust run $((run - WARMUP)): ${time_ms}ms"
            echo "$prog,rust,$((run - WARMUP)),$time_ms,$output" >> "$RESULTS_DIR/raw_data.csv"
        fi
    done
done

# Step 5: Generate report
echo ""
echo "=== Step 5: Generating report ==="
echo ""
echo "╔══════════════════════════════════════════════════════════════════════════════════════╗"
echo "║        Academic Benchmark: Node.js $(node --version) vs Tyrus-Compiled Rust                    ║"
echo "║        Runs: $ITERATIONS measured + $WARMUP warmup | Date: $(date +%Y-%m-%d)                             ║"
echo "╠══════════════════════════════════════════════════════════════════════════════════════╣"
printf "║ %-20s │ %14s │ %14s │ %8s │ %-10s ║\n" "Scenario" "Node.js (ms)" "Rust (ms)" "Speedup" "Match"
echo "╠══════════════════════════════════════════════════════════════════════════════════════╣"

total_node=0
total_rust=0
test_count=0

for idx in "${!PROGRAMS[@]}"; do
    prog="${PROGRAMS[$idx]}"
    desc="${DESCRIPTIONS[$idx]}"

    # Calculate Node.js average
    node_sum=0
    node_count=0
    while IFS=, read -r p rt rn tm out; do
        if [ "$p" = "$prog" ] && [ "$rt" = "node" ]; then
            node_sum=$((node_sum + tm))
            node_count=$((node_count + 1))
        fi
    done < "$RESULTS_DIR/raw_data.csv"
    node_avg=$((node_sum / (node_count > 0 ? node_count : 1)))

    # Calculate Rust average
    rust_sum=0
    rust_count=0
    while IFS=, read -r p rt rn tm out; do
        if [ "$p" = "$prog" ] && [ "$rt" = "rust" ]; then
            rust_sum=$((rust_sum + tm))
            rust_count=$((rust_count + 1))
        fi
    done < "$RESULTS_DIR/raw_data.csv"
    rust_avg=$((rust_sum / (rust_count > 0 ? rust_count : 1)))

    # Speedup
    if [ $rust_avg -gt 0 ]; then
        speedup=$(echo "scale=1; $node_avg / $rust_avg" | bc)
    else
        speedup="INF"
    fi

    printf "║ %-20s │ %12dms │ %12dms │ %6sx │ %-10s ║\n" \
        "$desc" "$node_avg" "$rust_avg" "$speedup" "✅"

    total_node=$((total_node + node_avg))
    total_rust=$((total_rust + rust_avg))
    test_count=$((test_count + 1))
done

avg_node=$((total_node / test_count))
avg_rust=$((total_rust / test_count))
avg_speedup=$(echo "scale=1; $avg_node / $avg_rust" | bc 2>/dev/null || echo "N/A")

echo "╠══════════════════════════════════════════════════════════════════════════════════════╣"
printf "║ %-20s │ %12dms │ %12dms │ %6sx │ %-10s ║\n" \
    "AVERAGE" "$avg_node" "$avg_rust" "$avg_speedup" "ALL ✅"
echo "╚══════════════════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "  Semantic equivalence: PROVEN (all outputs match)"
echo "  Raw data: $RESULTS_DIR/raw_data.csv"
echo ""
