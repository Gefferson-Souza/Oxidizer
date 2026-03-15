# 4. Mapeamento NestJS para Axum

Data: 2026-02-10
Status: Aceito

## Contexto

O framework alvo do projeto é o **NestJS** (TypeScript). Queremos que o código Rust gerado utilize um framework web robusto.
Escolhemos **Axum 0.7** (do ecosistema Tokio) por sua performance, ergonomia e compatibilidade com async.

## Decisão

Transformaremos Decorators do NestJS em rotas e extratores do Axum.

### Regras de Mapeamento:

1.  **Controllers:**
    - TS: `@Controller('users') class UsersController`
    - Rust: Uma função `pub fn router() -> Router` que agrupa as rotas.

2.  **Handlers (Métodos):**
    - TS: `@Get(':id') findOne(...)`
    - Rust: `pub async fn find_one(...)`
    - A rota é registrada no Router: `.route("/:id", get(find_one))`

3.  **Extractors (@Body, @Param, @Query):**
    - TS: `create(@Body() user: UserDto)` → Rust: `create(Json(user): Json<UserDto>)`
    - TS: `findOne(@Param('id') id: string)` → Rust: `find_one(Path(id): Path<String>)`
    - TS: `search(@Query('q') q: string)` → Rust: `search(Query(q): Query<String>)`

4.  **Injeção de Dependência:**
    - O `Dependency Injection` do NestJS é simulado via `State<Arc<Self>>` nos handlers.
    - O `Service` é instanciado no `main.rs` e passado via `.with_state(Arc::new(service))`.

5.  **Response Configuration:**
    - TS: `@HttpCode(201)` → Rust: `Result<(StatusCode, Json<T>), AppError>`
    - TS: `throw new NotFoundException("...")` → Rust: `return Err(AppError::NotFound("...".into()))`

6.  **Guards:**
    - TS: `@UseGuards(AuthGuard)` → Rust: `.layer(axum::middleware::from_fn(auth_guard_middleware))`
    - `canActivate(): boolean` → async middleware function

## Consequências

### Positivas

- Axum é extremamente rápido.
- O modelo de Extractors do Axum mapeia limpo para Decorators.
- Guards NestJS mapeiam para `axum::middleware::from_fn()` (tower middleware).
- `State<Arc<Self>>` permite compartilhar estado entre handlers sem `&mut self`.

### Negativas

- Interceptors e Pipes complexos do NestJS precisarão de mapeamentos adicionais.
- Dynamic modules (`forRoot`/`forAsync`) não suportados ainda.
