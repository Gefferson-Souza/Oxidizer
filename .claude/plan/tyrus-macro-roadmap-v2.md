# Tyrus Macro Roadmap v2 — Analise Honesta + Plano Completo

## Onde Estamos Hoje

| Metrica | Valor |
|---------|-------|
| Testes | 195 (81 equivalencia) |
| Crates | 10 (~9K linhas) |
| Cobertura NestJS real | ~15-25% |
| Pode compilar `nest new`? | SIM (basico) |
| Pode compilar projeto real com banco? | NAO |
| HTTP equivalence provado? | SIM (14/16 endpoints) |
| Benchmark feito? | SIM (9-14x throughput) |

## Analise Brutal: O que Falta

### TIER 1 — BLOQUEANTES (nenhum projeto real funciona sem)

| # | Feature | Dificuldade | Impacto | Rust Equivalent |
|---|---------|-------------|---------|-----------------|
| 1 | `Promise.all` | Media | Todo async service | `tokio::join!()` |
| 2 | `Map<K,V>` / `Set<T>` | Facil | Cache, lookup | `HashMap` / `HashSet` |
| 3 | `Date` / `Date.now()` | Media | Timestamps, audit | `chrono` |
| 4 | Destructuring em params `({name, email})` | Media | Pattern NestJS padrao | Pattern matching |
| 5 | Assignment operators (`-=`, `*=`, etc) | Facil | Aritmetica basica | `#left -= #right` |
| 6 | Object shorthand `{name}` → `{name: name}` | Facil | Todo objeto literal | Ja existe parcialmente |
| 7 | `process.env.VAR` / ConfigService | Media | Config de producao | `std::env::var()` |
| 8 | Validation (`class-validator`) | Alta | Input validation | `garde` crate |
| 9 | `typeof` / `instanceof` | Dificil | Type guards | Pattern matching |
| 10 | `response.json()` campo access | Alta | API consumption | `serde_json::Value["field"]` |

### TIER 2 — IMPORTANTES (maioria dos projetos usa)

| # | Feature | Dificuldade | Impacto |
|---|---------|-------------|---------|
| 11 | NestJS Pipes (ValidationPipe, ParseIntPipe) | Alta | Input transform |
| 12 | NestJS Interceptors | Muito Alta | Logging, transform |
| 13 | Exception Filters (@Catch) | Alta | Error handling |
| 14 | Custom decorators (createParamDecorator) | Muito Alta | Extensibilidade |
| 15 | `setTimeout` / `setInterval` | Facil | Timers |
| 16 | `crypto` (randomUUID, hash) | Media | Auth/security |
| 17 | `Buffer` | Media | Binary data |
| 18 | Async generators / `for await` | Alta | Streaming |

### TIER 3 — NICE TO HAVE (poucos projetos precisam)

| # | Feature | Dificuldade |
|---|---------|-------------|
| 19 | WebSocket gateway | Alta |
| 20 | Microservices transport | Muito Alta |
| 21 | TypeORM / Prisma → SQLx | Extrema |
| 22 | Symbol / Proxy / Reflect | Impossivel |
| 23 | Dynamic modules (forRoot/forAsync) | Muito Alta |
| 24 | RxJS Observables | Impossivel |

### FUNDAMENTALMENTE IMPOSSIVEL

1. **Reflect.getMetadata** — NestJS DI usa reflection runtime. Rust nao tem.
2. **RxJS Observables** — Interceptors NestJS usam Observable pipeline. Sem equivalente mecanico.
3. **Dynamic module loading** — `forRootAsync`, `useFactory` dependem de closures JS runtime.
4. **Prototype chain** — Mixins, method resolution via prototype. Sem equivalente em Rust.

## O que E Possivel (Horizonte Realista)

### Nivel 1: "Demo/PoC" (ONDE ESTAMOS HOJE)
- Projetos simples in-memory
- CRUD basico sem banco
- 5-6 endpoints
- Provado com benchmark

### Nivel 2: "Projeto Tutorial" (3-6 meses de trabalho)
Precisa: Map/Set, Promise.all, Date, destructuring, assignment ops, ConfigService
- Blog API basica
- Todo app
- Chat API simples
- **Ainda sem banco de dados**

### Nivel 3: "Microservice Isolado" (6-12 meses)
Precisa: Validation, Pipes, Interceptors, Exception Filters, crypto
- Service que processa dados
- API gateway simples
- Worker que consome fila
- **Banco de dados via API externa (nao ORM)**

### Nivel 4: "Producao Real" (12-24+ meses, se possivel)
Precisa: TypeORM/Prisma→SQLx, full module system, custom decorators
- E-commerce backend
- SaaS API
- **Requer decisoes arquiteturais fundamentais**

## Plano de Execucao (Priorizado por Impacto)

### Sprint 1: Quick Wins (1-2 semanas)
Corrigir bugs e features faceis que desbloqueiam muito:

- [ ] Fix assignment operators (`-=`, `*=`, `/=`, `%=`, `&=`, `|=`)
- [ ] Object shorthand properties `{name}` em contextos sem tipo
- [ ] `Map<K,V>` → `HashMap<K,V>` + `new Map()` → `HashMap::new()`
- [ ] `Set<T>` → `HashSet<T>` + `new Set()` → `HashSet::new()`
- [ ] `Date.now()` → `chrono::Utc::now().timestamp_millis()`
- [ ] Fix `this.method()` recursivo → `self.method()`

### Sprint 2: Core Async (2-3 semanas)
- [ ] `Promise.all([a, b])` → `tokio::join!(a, b)`
- [ ] `Promise.race([a, b])` → `tokio::select!(a, b)`
- [ ] `setTimeout(fn, ms)` → `tokio::time::sleep(Duration::from_millis(ms))`
- [ ] `setInterval` → `tokio::time::interval`

### Sprint 3: Type System (2-3 semanas)
- [ ] Destructuring em function params `({name, email}: Dto)`
- [ ] `typeof x === "string"` → match type guard
- [ ] `instanceof` → trait-based check
- [ ] Computed property names `{[key]: value}`
- [ ] Enum member comparison `HttpStatus.OK === 200`

### Sprint 4: NestJS Framework (3-4 semanas)
- [ ] `process.env.VAR` → `std::env::var("VAR")`
- [ ] `ConfigService.get("KEY")` → `std::env::var`
- [ ] `ValidationPipe` → `garde` validation
- [ ] `ParseIntPipe` → type coercion in extractor
- [ ] Exception Filters → Axum error handler

### Sprint 5: JSON & Data (2-3 semanas)
- [ ] `response.json()` → `serde_json::Value` with `["field"]` access
- [ ] `JSON.stringify(obj)` → `serde_json::to_string`
- [ ] `JSON.parse(str)` → `serde_json::from_str`
- [ ] `crypto.randomUUID()` → `uuid::Uuid::new_v4()`
- [ ] `crypto.createHash("sha256")` → `sha2::Sha256`

## Riscos e Mitigacoes

| Risco | Prob. | Mitigacao |
|-------|-------|----------|
| Escopo infinito | Alta | Foco no Oxidizable Standard, rejeitar patterns impossivel |
| Bugs silenciosos (codegen errado) | Alta | TODO teste deve ser equivalence (TS == Rust output) |
| Comunidade nao adota | Media | Focar em benchmark/LinkedIn primeiro, open source depois |
| Disco/memoria do dev | Media | Limpar caches, usar CI para builds pesados |
| Fadiga do desenvolvedor | Media | Celebrar cada milestone, postar progresso |
