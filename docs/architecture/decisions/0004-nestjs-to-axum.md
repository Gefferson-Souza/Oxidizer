# 4. NestJS to Axum Mapping

Date: 2026-02-10
Status: Accepted

## Context

The project's target framework is **NestJS** (TypeScript). We want the generated Rust code to use a robust web framework.
We chose **Axum 0.7** (from the Tokio ecosystem) for its performance, ergonomics, and async compatibility.

## Decision

We will transform NestJS Decorators into Axum routes and extractors.

### Mapping Rules:

1.  **Controllers:**
    - TS: `@Controller('users') class UsersController`
    - Rust: A `pub fn router() -> Router` function that groups the routes.

2.  **Handlers (Methods):**
    - TS: `@Get(':id') findOne(...)`
    - Rust: `pub async fn find_one(...)`
    - The route is registered in the Router: `.route("/:id", get(find_one))`

3.  **Extractors (@Body, @Param, @Query):**
    - TS: `create(@Body() user: UserDto)` → Rust: `create(Json(user): Json<UserDto>)`
    - TS: `findOne(@Param('id') id: string)` → Rust: `find_one(Path(id): Path<String>)`
    - TS: `search(@Query('q') q: string)` → Rust: `search(Query(q): Query<String>)`

4.  **Dependency Injection:**
    - NestJS `Dependency Injection` is simulated via `State<Arc<Self>>` in the handlers.
    - The `Service` is instantiated in `main.rs` and passed via `.with_state(Arc::new(service))`.

5.  **Response Configuration:**
    - TS: `@HttpCode(201)` → Rust: `Result<(StatusCode, Json<T>), AppError>`
    - TS: `throw new NotFoundException("...")` → Rust: `return Err(AppError::NotFound("...".into()))`

6.  **Guards:**
    - TS: `@UseGuards(AuthGuard)` → Rust: `.layer(axum::middleware::from_fn(auth_guard_middleware))`
    - `canActivate(): boolean` → async middleware function

## Consequences

### Positive

- Axum is extremely fast.
- Axum's Extractor model maps cleanly onto Decorators.
- NestJS Guards map to `axum::middleware::from_fn()` (tower middleware).
- `State<Arc<Self>>` allows sharing state across handlers without `&mut self`.

### Negative

- Complex NestJS Interceptors and Pipes will need additional mappings.
- Dynamic modules (`forRoot`/`forAsync`) are not yet supported.
