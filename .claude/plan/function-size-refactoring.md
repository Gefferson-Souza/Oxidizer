# Function Size Refactoring Plan

## Objective

Bring ALL 18 functions over 50 lines down to the limit. Zero exceptions. The compiler MUST eventually enforce this via `clippy::too_many_lines` and `clippy::cognitive_complexity`.

## Task Type
- [x] Backend (Rust refactoring — pure structural, no behavior changes)

## Principle

Every refactoring is a **pure structural split** — no behavior changes, no logic modifications. All 177 existing tests MUST pass after EACH step. If any test fails, the refactoring introduced a bug.

---

## Refactoring Phases (ordered by severity)

### Phase R1: `convert_method` (256 → ~5 functions)

**File:** `crates/tyrus_codegen/src/convert/class/method.rs`
**Current:** 256 lines, ONE function that handles decorators, params, return types, body, and output.

**Split into:**
| New Function | Lines | What it does |
|---|---|---|
| `extract_http_decorators(method) → (Option<String>, String, Option<u16>)` | ~25 | Scans `@Get/@Post/@HttpCode` decorators |
| `build_handler_params(method, is_handler) → Vec<TokenStream>` | ~40 | Builds params with `@Body/@Param/@Query` |
| `compute_return_type(method, is_handler, http_code) → TokenStream` | ~30 | Determines return type with Json/StatusCode wrapping |
| `build_handler_body(method, return_type, is_handler, http_code) → Vec<TokenStream>` | ~40 | Converts body with return handler closure |
| `convert_method(method, is_service) → (TokenStream, Option<route_info>)` | ~40 | Dispatcher that calls the above |

### Phase R2: `convert_constructor` (193 → ~4 functions)

**File:** `crates/tyrus_codegen/src/convert/class/constructor.rs`

**Split into:**
| New Function | Lines | What it does |
|---|---|---|
| `extract_constructor_params(constructor, deps, generics) → (Vec<TS>, HashSet)` | ~40 | Extract TsParamProp params |
| `extract_field_assignments(constructor, class_fields, deps) → Vec<TS>` | ~45 | Process this.field = value assignments |
| `build_di_constructor(constructor, fields, deps, generics) → TokenStream` | ~40 | Generate new_di() |
| `convert_constructor(struct_name, constructor, ...) → TokenStream` | ~30 | Dispatcher |

### Phase R3: stdlib handlers (math 158, array 133, string 124)

**Files:** `crates/tyrus_codegen/src/stdlib/{math,array,string}.rs`

**Pattern:** Each `handle()` function is a giant match. Extract each match arm into its own function:

```rust
// Before (158 lines):
pub(crate) fn handle(gen, method, args) -> Option<TokenStream> {
    match method {
        "max" => { /* 10 lines */ }
        "min" => { /* 10 lines */ }
        // ... 15 more arms
    }
}

// After (~30 lines):
pub(crate) fn handle(gen, method, args) -> Option<TokenStream> {
    match method {
        "max" => handle_max(gen, args),
        "min" => handle_min(gen, args),
        // ...
    }
}
fn handle_max(gen, args) -> Option<TokenStream> { /* 10 lines */ }
fn handle_min(gen, args) -> Option<TokenStream> { /* 10 lines */ }
```

### Phase R4: `process_fn_decl` (139 → ~3 functions)

**File:** `crates/tyrus_codegen/src/convert/fn_decl.rs`

**Split into:**
| New Function | Lines |
|---|---|
| `extract_fn_params(function) → Vec<TokenStream>` | ~30 |
| `build_fn_body(function, is_async, is_void) → Vec<TokenStream>` | ~40 |
| `process_fn_decl(n) → void` | ~40 |

### Phase R5: `process_import_decl` (127 → ~2 functions)

**File:** `crates/tyrus_codegen/src/convert/module.rs`

Split NestJS-specific import handling from generic import processing.

### Phase R6: `try_convert_array_method` (118 → ~3 functions)

**File:** `crates/tyrus_codegen/src/convert/expr/call.rs`

Split forEach, reduce, find handlers into separate functions.

### Phase R7: `map_type_core` (104 → ~2 functions)

**File:** `crates/tyrus_codegen/src/convert/type_mapper.rs`

Split keyword type mapping from reference type mapping.

### Phase R8: Smaller functions (84, 71, 66, 64, 59, 55, 53, 52, 51)

These are borderline. Extract the most complex logic into helpers.

---

## Implementation Order

| Step | What | Risk | Tests After |
|---|---|---|---|
| 1 | R3: stdlib (math, array, string) | Very Low — match arms are independent | 177 pass |
| 2 | R1: convert_method | Medium — complex function with closures | 177 pass |
| 3 | R2: convert_constructor | Medium — DI logic interleaved | 177 pass |
| 4 | R4: process_fn_decl | Low — clear separation points | 177 pass |
| 5 | R5-R8: remaining | Low — smaller changes | 177 pass |
| 6 | Enable clippy rules | None — validates all refactoring | 177 pass + clippy clean |

## Rules for Each Refactoring

1. **ONE function at a time** — don't batch
2. **Run `cargo nextest run --workspace` after EACH split** — must pass
3. **Run `cargo clippy --workspace` after EACH split** — must be clean
4. **No behavior changes** — pure structural move
5. **Extracted functions use `pub(crate)` or private `fn`** — never `pub`
6. **Parameter count ≤ 5** — use context structs if needed
7. **Each PR refactors ONE file** — separate branches per file

## After All Refactoring

Enable in `.cargo/config.toml`:
```toml
"-Wclippy::cognitive_complexity",
"-Wclippy::too_many_lines",
```

This makes the compiler enforce the rules going forward. Any function over 25 cognitive complexity or with excessive nesting will be a compile error.

## Key Files

| File | Current Max | Target Max |
|---|---|---|
| `class/method.rs` | 256 | ≤50 |
| `class/constructor.rs` | 193 | ≤50 |
| `stdlib/math.rs` | 158 | ≤30 (dispatcher) |
| `fn_decl.rs` | 139 | ≤50 |
| `stdlib/array.rs` | 133 | ≤30 (dispatcher) |
| `module.rs` | 127 | ≤50 |
| `stdlib/string.rs` | 124 | ≤30 (dispatcher) |
| `expr/call.rs` | 118 | ≤50 |
| `type_mapper.rs` | 104 | ≤50 |
| `expr/member.rs` | 84 | ≤50 |

## Risks and Mitigation

| Risk | Mitigation |
|---|---|
| Refactoring introduces behavior change | Run ALL 177 tests after each step |
| Extracted function has too many params | Use context struct `MethodContext { is_handler, http_code, ... }` |
| Snapshot tests fail from visibility change | Accept with `INSTA_UPDATE=always` |
| Closure captures break after extraction | Pass captured values as explicit params |
