# Plan: HTTP Benchmark Academico — NestJS vs Tyrus-Rust

## Task Type
- [x] Backend (→ Claude direct implementation)

## Objetivo

Benchmark completo e imparcial comparando NestJS (Node.js) vs Tyrus-compiled Rust (Axum), com rigor academico. Docker para isolamento, wrk2 para carga, metricas completas.

## Metodologia (baseada em TechEmpower + papers USENIX ATC / ACM SIGMETRICS)

### Principios
1. **Mesmo codigo fonte** — TypeScript identico, um roda no Node.js, outro transpilado pelo Tyrus
2. **Mesmos recursos** — Docker com --cpus, --memory, --cpuset-cpus identicos
3. **Carga separada** — load driver em CPUs diferentes do servidor
4. **Sem cherry-picking** — reportar TODOS os niveis de concorrencia
5. **Estatistica nao-parametrica** — mediana de 5 runs, Mann-Whitney U test

### Ferramentas
- **wrk2** — load generator com correcao de coordinated omission (Gil Tene, 2016)
- **docker stats** — RSS memory, CPU%
- **hyperfine** — cold start time

### Docker Resource Isolation
```bash
# Servidor (ambos recebem os mesmos recursos)
--cpus="2.0" --memory="512m" --memory-swap="512m" --cpuset-cpus="0,1"

# Load driver (CPUs separadas)
--cpus="4.0" --cpuset-cpus="2,3,4,5"
```

### Protocolo de Medicao
```
Para cada servidor (NestJS, Axum):
  Para cada concurrency (1, 16, 64, 128, 256):
    1. Iniciar container com resource limits
    2. Warmup: 30s (descartado)
    3. Estabilizacao: verificar CV < 5%
    4. Medicao: 60s com wrk2
    5. Capturar: docker stats (RSS, CPU%)
    6. Parar container
    7. Repetir 5x (runs independentes)
    8. Reportar: mediana dos 5 runs
```

### Metricas Coletadas

| Metrica | Unidade | Fonte |
|---------|---------|-------|
| Requests/second | req/s | wrk2 |
| Latencia p50 | ms | wrk2 --latency |
| Latencia p95 | ms | wrk2 --latency |
| Latencia p99 | ms | wrk2 --latency |
| Latencia p99.9 | ms | wrk2 --latency |
| RSS Memory (pico) | MB | docker stats |
| CPU utilization | % | docker stats |
| Transfer/sec | MB/s | wrk2 |
| Error rate | % | wrk2 non-2xx |
| Cold start time | ms | hyperfine |

### Workloads (3 tipos)

| Workload | Endpoint | Tipo | O que testa |
|----------|----------|------|-------------|
| **Plaintext** | GET / | I/O-bound | Framework overhead minimo |
| **Computation** | GET /calc/multiply | CPU-bound | Math operations |
| **String Processing** | POST /format/uppercase | Mixed | String manipulation + body parsing |

## Implementation Steps

### Step 1: Criar Dockerfiles
- `benchmarks/http/Dockerfile.nestjs` — Node.js 22 + NestJS app
- `benchmarks/http/Dockerfile.axum` — rust:1.75 + Axum binary
- `benchmarks/http/Dockerfile.loadgen` — wrk2 from source

### Step 2: Script de benchmark
- `benchmarks/http/run_benchmark.sh`
- Loop: foreach server, foreach concurrency, foreach workload
- Captura metricas em JSON/CSV
- 5 runs independentes por configuracao

### Step 3: Script de relatorio
- `benchmarks/http/generate_report.sh`
- Calcula mediana, IQR
- Gera tabelas comparativas
- Output: `benchmarks/http/results/REPORT.md`

### Step 4: docker-compose.yml
- Rede isolada (bench-net)
- Resource limits identicos
- Health checks antes de medir

## Riscos e Mitigacao

| Risco | Mitigacao |
|-------|----------|
| wrk2 nao instalado | Build from source no Dockerfile |
| Docker resource limits ignorados | Verificar com docker inspect |
| Node.js JIT nao aquecido | Warmup de 30s antes de medir |
| Resultados variam entre runs | 5 runs, mediana, Mann-Whitney U |
| Bias no relatorio | Reportar TODOS os dados, sem filtrar |

## Relatorio Final (Formato)

```
TYRUS HTTP BENCHMARK REPORT
===========================
Date: YYYY-MM-DD
Environment: [hardware specs]
Docker: [version]
Node.js: [version]
Rust: [version]

WORKLOAD: Plaintext (GET /)
+-----------+--------+--------+--------+--------+--------+--------+
| Framework | req/s  | p50(ms)| p95(ms)| p99(ms)| RSS(MB)| CPU(%) |
+-----------+--------+--------+--------+--------+--------+--------+
| NestJS    |  XXXXX |  X.XX  |  X.XX  |  X.XX  |  XXX   |  XX%   |
| Rust      |  XXXXX |  X.XX  |  X.XX  |  X.XX  |  XXX   |  XX%   |
+-----------+--------+--------+--------+--------+--------+--------+
| Ratio     |  X.Xx  |  X.Xx  |  X.Xx  |  X.Xx  |  X.Xx  |  X.Xx  |
+-----------+--------+--------+--------+--------+--------+--------+

[Repeat for each workload x each concurrency level]

STATISTICAL SIGNIFICANCE
- Mann-Whitney U test: p-value = X.XXXX (significant if < 0.05)
```

## Key Files

| File | Operation | Description |
|------|-----------|-------------|
| benchmarks/http/Dockerfile.nestjs | Create | NestJS container |
| benchmarks/http/Dockerfile.axum | Create | Rust container |
| benchmarks/http/docker-compose.yml | Create | Orchestration |
| benchmarks/http/run_benchmark.sh | Create | Measurement script |
| benchmarks/http/generate_report.sh | Create | Report generator |
