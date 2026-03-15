# Plan: Stress Benchmark — Cenarios Reais com async/await

## Task Type
- [x] Backend (→ Claude direct implementation)

## Objetivo

Benchmark intensivo simulando API real: requisicoes externas, processamento de dados,
manipulacao de strings, calculos matematicos, estado em memoria — tudo simultaneo.

## Limitacao Honesta

O Tyrus NAO suporta `response.json()` com acesso a campos JSON dinamicos (`data.field`).
Mas suporta `response.text()` + processamento de string, que e o que vamos usar.
Isso e documentado no relatorio como limitacao — sem esconder nada.

## Cenarios de Teste (6 endpoints)

### 1. GET /stress/compute — CPU Puro
Fibonacci(35) + Pi com 1000 iteracoes + soma de 1M numeros.
Testa: CPU-bound processing sem I/O.

### 2. GET /stress/strings — Manipulacao Massiva de Strings
Gera string de 10K chars, split, uppercase, lowercase, join, repeat.
Testa: alocacao de memoria + processamento de texto.

### 3. GET /stress/math — Calculos Complexos
100 chamadas a Math.sqrt, Math.pow, Math.sin, Math.cos em loop.
Testa: operacoes matematicas intensivas.

### 4. POST /stress/state — Estado com Mutex
Incrementa contador, armazena/recupera dados, operacoes atomicas.
Testa: contention no Arc<Mutex<T>> vs single-thread Node.js.

### 5. GET /stress/fetch — Async I/O externo
Faz fetch para API externa (jsonplaceholder), processa a resposta como texto.
Testa: async/await + I/O + processamento.
NOTA: tempo da API externa e igual para ambos — mede apenas overhead do framework.

### 6. GET /stress/combined — Tudo Junto
Computa + strings + math em uma unica request.
Testa: carga mista simulando handler real.

## Protocolo de Benchmark

1. Warmup: 15s por servidor
2. Medicao: 30s por teste, c=128
3. Simultaneo: bombardier com multiplas rotas ao mesmo tempo
4. Metricas: req/s, p50/p95/p99 latency, RSS, CPU%
5. Verificacao: comparar respostas para garantir equivalencia

## Relatorio

Narrativo + tabelas. Explicar cada resultado, por que um e mais rapido,
onde o Node.js tem vantagens, limitacoes do teste.

## Key Files

| File | Operation | Description |
|------|-----------|-------------|
| /tmp/stress-app/src/app.service.ts | Create | Service com 6 endpoints |
| /tmp/stress-app/src/app.controller.ts | Create | Controller mapeando rotas |
| /tmp/stress-app/src/app.module.ts | Create | Module |
