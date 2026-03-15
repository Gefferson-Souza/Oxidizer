#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"
DURATION="${BENCH_DURATION:-30s}"
WARMUP="${BENCH_WARMUP:-10s}"
CONCURRENCY_LEVELS="${BENCH_CONCURRENCY:-16 64 128 256}"
RUNS="${BENCH_RUNS:-3}"

mkdir -p "$RESULTS_DIR"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  TYRUS HTTP BENCHMARK — Academic Grade                      ║"
echo "║  NestJS (Node.js) vs Rust (Axum/Tyrus-compiled)             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Environment info
echo "=== ENVIRONMENT ==="
echo "Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "Host: $(uname -srm)"
echo "CPU: $(grep 'model name' /proc/cpuinfo 2>/dev/null | head -1 | cut -d: -f2 | xargs || echo 'unknown')"
echo "Cores: $(nproc)"
echo "RAM: $(free -h | awk '/Mem:/ {print $2}')"
echo "Docker: $(docker --version | cut -d' ' -f3)"
echo "Duration: $DURATION | Warmup: $WARMUP | Runs: $RUNS"
echo "Concurrency levels: $CONCURRENCY_LEVELS"
echo ""

# Build containers
echo "=== BUILDING CONTAINERS ==="
cd "$SCRIPT_DIR"
docker compose build 2>&1 | tail -5
echo ""

# Start both servers
echo "=== STARTING SERVERS ==="
docker compose up -d
echo "Waiting for health checks..."
sleep 10

# Verify both are healthy
NEST_OK=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:3100/ 2>/dev/null || echo "000")
AXUM_OK=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:3101/ 2>/dev/null || echo "000")

if [ "$NEST_OK" != "200" ] || [ "$AXUM_OK" != "200" ]; then
    echo "ERROR: Servers not ready (NestJS=$NEST_OK, Axum=$AXUM_OK)"
    docker compose logs
    docker compose down
    exit 1
fi

echo "NestJS: OK ($NEST_OK) | Axum: OK ($AXUM_OK)"
echo ""

# Verify identical responses
echo "=== RESPONSE EQUIVALENCE CHECK ==="
NEST_RESP=$(curl -s http://127.0.0.1:3100/)
AXUM_RESP=$(curl -s http://127.0.0.1:3101/)
if [ "$NEST_RESP" = "$AXUM_RESP" ]; then
    echo "GET /: IDENTICAL ('$NEST_RESP')"
else
    echo "WARNING: GET / differs — NestJS='$NEST_RESP' Axum='$AXUM_RESP'"
fi

NEST_CALC=$(curl -s http://127.0.0.1:3100/calc/multiply)
AXUM_CALC=$(curl -s http://127.0.0.1:3101/calc/multiply)
if [ "$NEST_CALC" = "$AXUM_CALC" ]; then
    echo "GET /calc/multiply: IDENTICAL ('$NEST_CALC')"
else
    echo "WARNING: GET /calc/multiply differs"
fi

NEST_FMT=$(curl -s -X POST http://127.0.0.1:3100/format/uppercase)
AXUM_FMT=$(curl -s -X POST http://127.0.0.1:3101/format/uppercase)
if [ "$NEST_FMT" = "$AXUM_FMT" ]; then
    echo "POST /format/uppercase: IDENTICAL ('$NEST_FMT')"
else
    echo "WARNING: POST /format/uppercase differs"
fi
echo ""

# Collect container stats (idle baseline)
echo "=== IDLE RESOURCE USAGE ==="
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}" bench-nestjs bench-axum
echo ""

# Benchmark function
run_bench() {
    local name="$1"
    local port="$2"
    local endpoint="$3"
    local method="$4"
    local concurrency="$5"
    local run_num="$6"
    local label="${name}_c${concurrency}_run${run_num}"

    local url="http://127.0.0.1:${port}${endpoint}"

    # Warmup (discarded)
    if [ "$method" = "GET" ]; then
        docker run --rm --network=host williamyeh/wrk -t2 -c"$concurrency" -d"$WARMUP" "$url" > /dev/null 2>&1
    else
        docker run --rm --network=host williamyeh/wrk -t2 -c"$concurrency" -d"$WARMUP" -s <(echo "wrk.method = \"$method\"") "$url" > /dev/null 2>&1 || true
    fi

    # Measurement
    local result
    if [ "$method" = "GET" ]; then
        result=$(docker run --rm --network=host williamyeh/wrk -t4 -c"$concurrency" -d"$DURATION" --latency "$url" 2>&1)
    else
        result=$(docker run --rm --network=host williamyeh/wrk -t4 -c"$concurrency" -d"$DURATION" --latency -s <(echo "wrk.method = \"$method\"") "$url" 2>&1 || docker run --rm --network=host williamyeh/wrk -t4 -c"$concurrency" -d"$DURATION" --latency "$url" 2>&1)
    fi

    echo "$result" > "$RESULTS_DIR/${label}.txt"

    # Extract metrics
    local reqs=$(echo "$result" | grep "Requests/sec" | awk '{print $2}')
    local lat50=$(echo "$result" | grep "50%" | awk '{print $2}' | head -1)
    local lat99=$(echo "$result" | grep "99%" | awk '{print $2}' | head -1)
    local transfer=$(echo "$result" | grep "Transfer/sec" | awk '{print $2}')

    # Container stats during load
    local stats=$(docker stats --no-stream --format "{{.CPUPerc}}|{{.MemUsage}}" "bench-${name,,}" 2>/dev/null || echo "N/A|N/A")
    local cpu=$(echo "$stats" | cut -d'|' -f1)
    local mem=$(echo "$stats" | cut -d'|' -f2)

    printf "  %-8s c=%-4s run=%s | %10s req/s | p50=%-8s p99=%-8s | CPU=%-7s MEM=%s\n" \
        "$name" "$concurrency" "$run_num" "$reqs" "$lat50" "$lat99" "$cpu" "$mem"

    echo "${name},${endpoint},${concurrency},${run_num},${reqs},${lat50},${lat99},${cpu},${mem},${transfer}" >> "$RESULTS_DIR/raw_data.csv"
}

# CSV header
echo "framework,endpoint,concurrency,run,req_per_sec,p50_latency,p99_latency,cpu_percent,memory,transfer_per_sec" > "$RESULTS_DIR/raw_data.csv"

# Workloads
WORKLOADS=(
    "GET:/:Plaintext"
    "GET:/calc/multiply:Computation"
    "POST:/format/uppercase:StringProcessing"
)

for workload in "${WORKLOADS[@]}"; do
    IFS=':' read -r method endpoint label <<< "$workload"
    echo "=== WORKLOAD: $label ($method $endpoint) ==="
    echo ""

    for concurrency in $CONCURRENCY_LEVELS; do
        for run in $(seq 1 "$RUNS"); do
            run_bench "NestJS" 3100 "$endpoint" "$method" "$concurrency" "$run"
            run_bench "Axum" 3101 "$endpoint" "$method" "$concurrency" "$run"
        done
        echo ""
    done
done

# Final resource usage under rest
echo "=== FINAL RESOURCE USAGE ==="
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}\t{{.BlockIO}}" bench-nestjs bench-axum
echo ""

# Cleanup
echo "=== CLEANUP ==="
docker compose down 2>/dev/null
echo ""

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  BENCHMARK COMPLETE                                         ║"
echo "║  Results saved to: benchmarks/http/results/                 ║"
echo "║  Raw data: results/raw_data.csv                             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
