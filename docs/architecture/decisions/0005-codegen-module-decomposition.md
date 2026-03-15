# 5. Decomposition of Monolithic Codegen Modules

Date: 2026-03-12
Status: Accepted

## Context

The `tyrus_codegen` crate contained two monolithic files that violated the project's strict quality thresholds:

- **`func.rs`** (1144 lines): Handled all expression conversion, statement processing, function declarations, and helper utilities in a single file.
- **`class.rs`** (1048 lines): Handled all class-related transpilation including constructors, methods, NestJS decorators, Axum routing, and self-mutation analysis.

Additionally, the orchestrator `lib.rs` (508 lines) mixed public API, pipeline logic, scaffolding, and formatting in one file.

These files exceeded the 400-line threshold, had high cognitive complexity, and made isolated testing difficult.

## Decision

### func.rs Decomposition (Chunk 2)

Split into 10 focused modules:

| Module | Responsibility |
|--------|---------------|
| `helpers.rs` | Case conversion utilities and type detection helpers |
| `stmt.rs` | Statement conversion logic |
| `fn_decl.rs` | Function declaration processing |
| `expr/mod.rs` | Expression dispatcher |
| `expr/binary.rs` | Binary operator transpilation |
| `expr/call.rs` | Function/method calls, array methods, axios/fetch |
| `expr/member.rs` | Property access and mutex state |
| `expr/arrow.rs` | Arrow functions to closures |
| `expr/literal.rs` | Literals, objects, arrays, template literals |
| `expr/misc.rs` | Assignments, updates, optional chaining |

### stmt.rs Further Decomposition (Phase 6)

Added sub-modules as new statement types were implemented:

| Module | Responsibility |
|--------|---------------|
| `stmt/try_catch.rs` | try-catch → Result matching pattern |

### class.rs Decomposition (Chunk 3 + Phase 6-7)

Split into 6 focused modules under `class/`:

| Module | Responsibility |
|--------|---------------|
| `class/mod.rs` | Class dispatcher + property conversion |
| `class/constructor.rs` | Constructor transpilation + DI detection |
| `class/method.rs` | Method transpilation + decorator parsing |
| `class/getter_setter.rs` | Getter/setter → accessor methods |
| `class/routing.rs` | Axum router generation + @UseGuards middleware |
| `class/mutation.rs` | Static self-mutation analysis |

### Orchestrator Decomposition (Chunk 3)

Split `lib.rs` (508 lines) into 4 modules:

| Module | Responsibility |
|--------|---------------|
| `lib.rs` | Slim public API (58 lines) |
| `pipeline.rs` | Core multi-file build orchestration |
| `scaffold.rs` | Project scaffolding (main.rs, Cargo.toml, mod.rs) |
| `format.rs` | Code formatting + AppError generation |

### type_mapper.rs Deduplication (Chunk 3)

Consolidated `map_ts_type` and `map_inner_type` into a single `map_type_core` function, reducing from 304 to 257 lines.

## Consequences

### Positive

- All files are under 400 lines (most under 200).
- Each module has a single, clear responsibility.
- Enables targeted unit testing of individual transpilation concerns.
- Reduces cognitive load when navigating the codebase.
- The original `func.rs` and monolithic `class.rs` were deleted entirely.

### Negative

- More files to navigate (mitigated by clear naming conventions).
- Cross-module function calls require explicit imports (mitigated by `pub(crate)` visibility).
