# 2. Async/Await Transpilation Strategy

Date: 2026-02-10
Status: Accepted

## Context

TypeScript and Rust have different concurrency models.

- **TS:** Single-threaded Event Loop, `Promise<T>`.
- **Rust:** Multi-threaded (potentially), `Future<Output=T>`, requires a Runtime (Tokio).

We need to define how to transform `async` functions from TS into Rust in a way that is compatible with the `axum`/`tokio` ecosystem.

## Decision

We will map TypeScript's `async/await` directly to Rust's `async/.await` syntax, using the `tokio` crate as the runtime.

### Mapping Rules:

1.  **Function Signature:**
    - TS: `async function foo(): Promise<string>`
    - Rust: `pub async fn foo() -> Result<String, AppError>`
    - _Note:_ All async functions must return `Result` for error propagation (`?`), even if the TS version does not explicitly throw exceptions.

2.  **Promise Unwrapping:**
    - The `Promise<T>` return type is "unwrapped" to `T` (inside the `Result`).

3.  **Await Expression:**
    - TS: `await foo()`
    - Rust: `foo().await?`
    - _Note:_ The `?` operator is added automatically to handle errors, assuming any Future can fail (standard for I/O).

4.  **Runtime:**
    - The generated binary will depend on `#[tokio::main]`.

## Consequences

### Positive

- Generated code is highly idiomatic and readable.
- Native integration with crates like `reqwest` and `sqlx`, which are `async`.
- Superior performance to Node.js's Event Loop for I/O-bound tasks.

### Negative

- Forces the use of `tokio` (increases binary size).
- `async` in traits (Rust) is still complex (although solved in Rust 1.75+ with RPITIT, edge cases may still exist).
