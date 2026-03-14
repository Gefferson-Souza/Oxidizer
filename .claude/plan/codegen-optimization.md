# Codegen Optimization Plan: Closing the Gap with Hand-Written Rust

## Problem Statement

Benchmark results show Tyrus-generated Rust has 2 major performance gaps vs hand-written:

| Cenário | Tyrus/Hand | Root Cause |
|---------|-----------|------------|
| Text Processing | **15.0x slower** | Nested `format!("{}{}", ...)` for string concatenation |
| Matrix Compute | **11.0x slower** | No pre-computation, redundant trig calls per iteration |
| Statistics | **1.6x slower** | `.iter().cloned()` + no `Vec::with_capacity` |
| Sorting | **1.5x slower** | `sort_by` instead of `sort_unstable_by` |
| Data Pipeline | **0.8x** (faster!) | Already optimal |
| Accumulation | **0.8x** (faster!) | Already optimal |

## Task Type
- [x] Backend (Rust code generation optimization)

## Optimization Phases

### Phase A: String Concatenation Fix (eliminates 15x overhead)
**Impact: CRITICAL — Text Processing 15x → ~2x**

**Root cause:** `binary.rs:22` generates `format!("{}{}", left, right)` for every `+` between strings. When TS has `a + b + c + d`, this becomes:
```rust
format!("{}{}", format!("{}{}", format!("{}{}", a, b), c), d)
```
Each `format!` allocates a new String. 4 concatenations = 4 allocations.

**Fix:** Flatten string concat chains into a single `format!()` with multiple args:
```rust
// Before (current): 4 allocations
format!("{}{}", format!("{}{}", format!("{}{}", a, b), c), d)

// After (optimized): 1 allocation
format!("{}{}{}{}", a, b, c, d)
```

**Implementation:**
1. In `convert_bin_expr`, when encountering `Add` with strings:
   - Walk the AST to collect ALL chained `+` operands (recursive left-descent)
   - Emit a single `format!("{}{}{}", arg1, arg2, arg3, ...)` with N `{}` placeholders
2. Only flatten when ALL operands are string-concatenation `Add` nodes

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/expr/binary.rs`
- Add helper: `fn collect_string_concat_chain(&self, expr: &Expr) -> Vec<TokenStream>`

**Test:**
- Existing text processing benchmark should drop from 150ms to ~20ms
- Equivalence tests must still pass

---

### Phase B: Iterator Chain Optimization (eliminates 1.5-2x overhead)
**Impact: HIGH — Statistics 1.6x → ~1.1x, Pipeline stays optimal**

**Root cause:** `call.rs` generates:
```rust
obj.clone().into_iter().filter(...).collect::<Vec<_>>()
   .clone().into_iter().map(...).collect::<Vec<_>>()
   .iter().cloned().fold(...)
```

Each `.collect::<Vec<_>>()` allocates a NEW Vec. A chain of `filter→map→reduce` creates 2 intermediate Vecs.

**Fix:** Detect chains of `filter→map→reduce` and fuse them into a single iterator:
```rust
// Before: 3 allocations (Vec for filter result, Vec for map result)
obj.clone().into_iter().filter(f).collect::<Vec<_>>()
   .clone().into_iter().map(m).collect::<Vec<_>>()
   .iter().cloned().fold(init, r)

// After: 0 intermediate allocations
obj.iter().filter(f).map(m).fold(init, r)
```

**Implementation:**
1. In `try_convert_array_method` in `call.rs`, detect when the callee is itself an array method call
2. When chained (e.g., `.filter().map()`), don't emit `.collect::<Vec<_>>()` for the intermediate step
3. Only `.collect()` at the final step (or not at all if terminal is `reduce`/`forEach`)

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/expr/call.rs`

**Risk:** Medium — chained method detection requires walking the MemberExpr AST

---

### Phase C: Vec::with_capacity (eliminates allocation overhead)
**Impact: MEDIUM — ~10-20% improvement across all array-heavy tests**

**Root cause:** Tyrus generates `let mut data = vec![];` then `data.push(...)` in a loop. The Vec resizes ~17 times for 100K elements (doubling strategy). Hand-written Rust uses `Vec::with_capacity(size)`.

**Fix:** When a `let mut arr = []` is followed by a `while` loop with `arr.push()`, detect the loop bound and emit `Vec::with_capacity(bound)`.

**Implementation:**
1. In `convert_var_decl` (stmt/var_decl.rs), when init is `[]`:
   - Look ahead at the next statements for a while-loop pattern
   - If the loop pushes to this array and has a clear bound variable, emit `Vec::with_capacity(bound as usize)`
2. Simpler alternative: always emit `Vec::with_capacity(1024)` for `let mut arr = []` (less precise but safe)

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/stmt/var_decl.rs`

**Risk:** Low — `with_capacity` is always correct (just pre-allocates)

---

### Phase D: Sort Optimization (eliminates 1.5x overhead)
**Impact: LOW-MEDIUM — Sorting 1.5x → ~1.0x**

**Root cause:** Tyrus generates `sort_by(|a, b| a.partial_cmp(b).unwrap_or(...))` which is stable sort. Hand-written uses `sort_unstable_by` which is faster for primitives (no stability guarantee needed).

**Fix:** Use `sort_unstable_by` for numeric arrays.

**Implementation:**
1. In `stdlib/array.rs`, change `sort` handler:
   ```rust
   // Before
   obj.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
   // After
   obj.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
   ```

**Files:**
- Modify: `crates/tyrus_codegen/src/stdlib/array.rs`

**Risk:** Very low — 1-line change, sort_unstable_by is always faster

---

### Phase E: Redundant Clone Elimination (reduces overhead across all tests)
**Impact: MEDIUM — ~20-30% improvement on array-heavy operations**

**Root cause:** `call.rs` generates `.clone().into_iter()` for EVERY array method. Many of these clones are unnecessary when the array is only used once.

**Fix (simple):** Replace `.clone().into_iter()` with `.iter()` where possible:
```rust
// Before
obj.clone().into_iter().filter(|v| ...)

// After (for non-consuming operations)
obj.iter().filter(|v| ...)
```

**Implementation:**
1. For `filter`, `map`, `forEach`, `some`, `every`, `find`, `reduce` — use `.iter()` (borrows, doesn't consume)
2. Keep `.into_iter()` only for operations that genuinely need ownership (rare in the Oxidizable Standard)

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/expr/call.rs`

**Note:** `.iter().cloned()` was already applied for forEach/reduce/find/some/every in Phase 5. But `filter` and `map` still use `.clone().into_iter()`.

---

### Phase F: Re-run Benchmarks
**Impact: Validate all optimizations**

After implementing Phases A-E:
1. Re-transpile all 6 benchmark programs
2. Re-compile with release+LTO
3. Run three-way comparison (Node vs Tyrus vs Hand)
4. Document delta: before → after per optimization

---

## Expected Results After Optimization

| Cenário | Before (Tyrus/Hand) | After (estimated) | Improvement |
|---------|--------------------|--------------------|-------------|
| Text Processing | 15.0x | ~2.0x | **7.5x better** |
| Matrix | 11.0x | ~3.0x | **3.7x better** (needs pre-computation too) |
| Statistics | 1.6x | ~1.1x | **1.5x better** |
| Sorting | 1.5x | ~1.0x | **1.5x better** |
| Data Pipeline | 0.8x | ~0.7x | Stays optimal |
| Accumulation | 0.8x | ~0.7x | Stays optimal |

## Implementation Steps (ordered by impact)

| Step | Phase | Impact | Effort | Risk |
|------|-------|--------|--------|------|
| 1 | A: String concat flatten | **CRITICAL** (15x→2x) | Medium | Low |
| 2 | D: sort_unstable_by | **LOW** (1.5x→1.0x) | Trivial | None |
| 3 | E: Remove .clone().into_iter() | **MEDIUM** (20-30%) | Low | Low |
| 4 | B: Iterator fusion | **HIGH** (1.6x→1.1x) | High | Medium |
| 5 | C: Vec::with_capacity | **MEDIUM** (10-20%) | Medium | Low |
| 6 | F: Re-run benchmarks | Validation | Low | None |

## Key Files

| File | Operation | Description |
|------|-----------|-------------|
| `crates/tyrus_codegen/src/convert/expr/binary.rs` | Modify | String concat flattening (Phase A) |
| `crates/tyrus_codegen/src/convert/expr/call.rs` | Modify | Iterator chain fusion + clone removal (Phase B, E) |
| `crates/tyrus_codegen/src/stdlib/array.rs` | Modify | sort_unstable_by (Phase D) |
| `crates/tyrus_codegen/src/convert/stmt/var_decl.rs` | Modify | Vec::with_capacity (Phase C) |
| `benchmarks/academic/scripts/run_all.sh` | Run | Re-benchmark (Phase F) |

## Risks and Mitigation

| Risk | Mitigation |
|------|------------|
| String flatten breaks non-string `+` | Only flatten when `is_string_expr` returns true for chain root |
| Iterator fusion breaks closure ownership | Keep `.cloned()` in closures that need owned values |
| Vec::with_capacity wrong size | Use `as usize` cast; for unknown sizes, keep `vec![]` |
| sort_unstable_by changes sort order | Same comparator, just different algorithm (no stability) |
| Existing 168 tests regress | Run full suite after each phase |
