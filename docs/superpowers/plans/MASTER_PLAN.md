# Tyrus Master Plan — Unified Roadmap

> Central plan governing all development. Macro phases → Micro milestones → Nano tasks.
> Every PR maps to a nano task. Every nano task has an equivalence test proving it works.

**Principle:** Nothing advances without semantic equivalence proof. Every new feature must pass `assert_output_equivalent()`.

---

## Current State (2026-03-13)

| Metric | Value |
|--------|-------|
| Tests passing | 179 (71 equivalence + 7 CLI + 8 IR + 73 integration + 9 codegen + 4 common + 1 trybuild + 1 skipped) |
| Equivalence tests | 71 (proving TS↔Rust output identity) |
| Array spread | `[...a, ...b]` → `a.iter().cloned().chain(b.iter().cloned()).collect()` |
| Static methods | `Class.method()` → `Class::method()` (associated functions) |
| Type assertions | `as Type` → no-op (4 SWC variants handled) |
| Numeric enums | `enum Dir { Up = 0 }` → `#[repr(i32)] enum` with Display |
| Supported expressions | 16+ types |
| Control flow | while, for, for-of, do-while, switch, if/else, try-catch |
| Top-level statements | const, let, expressions, console.log — auto-wrapped in fn main() |
| String methods | 16 (includes, replace, split, toUpperCase, toLowerCase, trim, startsWith, endsWith, toString, substring, charAt, indexOf, repeat, slice, padStart, padEnd) |
| Array methods | 15 (map, filter, forEach, find, some, every, reduce, join, includes, push, indexOf, slice, concat, reverse, pop, sort, shift, flat, flatMap) |
| Math functions | 15 (max, min, round, floor, ceil, abs, random, pow, sqrt, log, trunc, sign, sin, cos, tan) |
| Math constants | 2 (PI, E) |
| Console methods | 5 (log, error, warn, info, debug) |
| Blocked by analyzer | 4 constructs (for-in, delete, with, labeled) |
| CLI commands | 4 (check, build, compile, run) |
| Analyzer lint rules | 7 (var, any, eval, for-in, delete, with, labeled) |
| Unsupported APIs detected | 11 (DOM, timers, require, XMLHttpRequest, etc.) |
| IR types defined | 4 (TyrusType, TyrusExpr, TyrusStmt, TyrusDecl) |
| Known bugs | 4 (optional chaining double-Option, try-catch finally skipped on success #36, side-effect try early return #37, return in loop inside try #38) |

---

## MACRO PHASES

```
Phase 1 ✅ Foundation     (Milestones 1-8)   — Core transpilation, types, NestJS
Phase 2 ✅ Quality         (Milestones 9-12)  — Strict rules, decomposition, test suite
Phase 3 ✅ Equivalence     (Milestone 13A)    — Prove output identity for basics
Phase 4 ✅ Control Flow    (Milestone 13B)    — Unlock blocked constructs + bug fixes
Phase 5 ✅ Stdlib Complete (Milestone 14)     — Full JS/TS method coverage (ALL methods done)
Phase 5.5 ✅ Architecture  (CLI+IR+Analyzer)  — Branded CLI, typed IR, expanded analyzer
Phase 6.0 ✅ Infrastructure (prettyplease, thiserror 2.0, trybuild) — See docs/superpowers/plans/2026-03-13-nestjs-full-transpilation-roadmap.md
Phase 6 ✅ Advanced TS     (Milestone 15)     — Class inheritance, static, enums, type assertions, spread
Phase 7 📋 Academic        (Milestone 16)     — Benchmarks, formal spec, paper
```

---

## Phase 4: Control Flow Expansion (Milestone 13B)

**Goal:** Unlock 6 blocked control flow constructs and fix remaining bugs. Coverage: ~70% → ~90%.

### Micro 4.1: Bug Fixes

| Nano | Task | Files | Equivalence Test |
|------|------|-------|-----------------|
| 4.1.1 | Fix optional chaining `Some()` wrapper | `expr/misc.rs` | `obj?.name` returns correct value |
| 4.1.2 | Fix `.find()` closure type mismatch (`&&f64`) | `expr/call.rs` | `[1,2,3].find(n => n > 1)` |
| 4.1.3 | Fix parenthesized expression precedence | `expr/mod.rs` | `(2 + 3) * 4` = 20, not 14 |

### Micro 4.2: Unblock Analyzer

| Nano | Task | Files | Equivalence Test |
|------|------|-------|-----------------|
| 4.2.1 | Remove analyzer rejection for `for-of` | `lints.rs` | `for (const x of arr) { console.log(x) }` |
| 4.2.2 | Remove analyzer rejection for `for-in` | `lints.rs` | `for (const k in obj) { console.log(k) }` |
| 4.2.3 | Remove analyzer rejection for `do-while` | `lints.rs` | `do { i++ } while (i < 5)` |
| 4.2.4 | Remove analyzer rejection for `for` | `lints.rs` | `for (let i=0; i<5; i++) { console.log(i) }` |

### Micro 4.3: New Control Flow Codegen

| Nano | Task | Files | Equivalence Test |
|------|------|-------|-----------------|
| 4.3.1 | Traditional `for` loop → `while` | `stmt.rs` | `for(let i=0; i<n; i++) { sum += i }` |
| 4.3.2 | `do-while` → `loop { ... if !cond { break } }` | `stmt.rs` | `do { x++ } while (x < 10)` |
| 4.3.3 | `switch` → `match` | `stmt.rs`, `lints.rs` | `switch(x) { case "a": ... }` |
| 4.3.4 | `try-catch` → `match` on Result | `stmt.rs`, `lints.rs` | `try { ok() } catch(e) { err() }` |

---

## Phase 5: Stdlib Complete (Milestone 14)

**Goal:** Cover the most-used JS/TS methods. Each nano task adds ONE method + equivalence test.

### Micro 5.1: String Methods

| Nano | Method | Rust Mapping | Priority | Status |
|------|--------|-------------|----------|--------|
| 5.1.1 | `substring(start, end)` | `s[start..end].to_string()` | HIGH | ✅ |
| 5.1.2 | `charAt(i)` | `s.chars().nth(i)` | HIGH | ✅ |
| 5.1.3 | `indexOf(substr)` | `s.find(substr)` | HIGH | ✅ |
| 5.1.4 | `repeat(n)` | `s.repeat(n)` | MEDIUM | ✅ |
| 5.1.5 | `padStart(len, fill)` | `format!("{:>width$}", s)` | LOW | ✅ |
| 5.1.6 | `padEnd(len, fill)` | `format!("{:<width$}", s)` | LOW | ✅ |
| 5.1.7 | `slice(start, end)` | `s[start..end].to_string()` | MEDIUM | ✅ |

### Micro 5.2: Array Methods

| Nano | Method | Rust Mapping | Priority | Status |
|------|--------|-------------|----------|--------|
| 5.2.1 | `indexOf(item)` | `.iter().position(\|x\| x == &item)` | HIGH | ✅ |
| 5.2.2 | `slice(start, end)` | `[start..end].to_vec()` | HIGH | ✅ |
| 5.2.3 | `concat(other)` | `.iter().chain(other.iter()).cloned().collect()` | MEDIUM | ✅ |
| 5.2.4 | `sort()` | `.sort_by(\|a,b\| a.partial_cmp(b).unwrap_or(Ordering::Equal))` | MEDIUM | ✅ |
| 5.2.5 | `reverse()` | `.reverse()` | LOW | ✅ |
| 5.2.6 | `pop()` | `.pop()` | LOW | ✅ |
| 5.2.7 | `shift()` | `.remove(0)` | LOW | ✅ |
| 5.2.8 | `flat()` / `flatMap()` | `.into_iter().flatten().collect()` | LOW | ✅ |

### Micro 5.3: Math Functions

| Nano | Method | Rust Mapping | Priority | Status |
|------|--------|-------------|----------|--------|
| 5.3.1 | `Math.pow(base, exp)` | `base.powf(exp)` | HIGH | ✅ |
| 5.3.2 | `Math.sqrt(x)` | `x.sqrt()` | HIGH | ✅ |
| 5.3.3 | `Math.PI` | `std::f64::consts::PI` | HIGH | ✅ |
| 5.3.4 | `Math.E` | `std::f64::consts::E` | MEDIUM | ✅ |
| 5.3.5 | `Math.log(x)` | `x.ln()` | MEDIUM | ✅ |
| 5.3.6 | `Math.sin/cos/tan(x)` | `x.sin()` / `.cos()` / `.tan()` | LOW | ✅ |
| 5.3.7 | `Math.sign(x)` | custom zero-check + `signum()` | LOW | ✅ |
| 5.3.8 | `Math.trunc(x)` | `x.trunc()` | LOW | ✅ |

### Micro 5.X: Infrastructure — String/Array Disambiguation

| Nano | Task | Status |
|------|------|--------|
| 5.X.1 | `string_vars` tracking via `RefCell<HashSet>` in `RustGenerator` | ✅ |
| 5.X.2 | Type annotation detection in `convert_stmt` for `: string` variables | ✅ |
| 5.X.3 | Dispatcher checks `string_vars` for Ident expressions | ✅ |

### Micro 5.4: Object Methods

| Nano | Method | Rust Mapping | Priority | Status |
|------|--------|-------------|----------|--------|
| 5.4.1 | `Object.keys(obj)` | `obj.keys().cloned().collect::<Vec<_>>()` | HIGH | ✅ |
| 5.4.2 | `Object.values(obj)` | `obj.values().cloned().collect::<Vec<_>>()` | HIGH | ✅ |
| 5.4.3 | `Object.entries(obj)` | `obj.iter().collect::<Vec<_>>()` | MEDIUM | ✅ |

### Micro 5.5: Console Methods

| Nano | Method | Rust Mapping | Priority | Status |
|------|--------|-------------|----------|--------|
| 5.5.1 | `console.warn()` | `eprintln!(...)` | LOW | ✅ |
| 5.5.2 | `console.info()` | `println!(...)` | LOW | ✅ |
| 5.5.3 | `console.debug()` | `println!(...)` | LOW | ✅ |

---

## Phase 6: Advanced TypeScript (Milestone 15)

**Goal:** Support advanced TS patterns that are common in real codebases.

### Micro 6.1: Class Features

| Nano | Feature | Description |
|------|---------|-------------|
| 6.1.1 | Class inheritance | `extends` → trait impl + composition |
| 6.1.2 | Getters/Setters | `get x()` → method, `set x(v)` → method |
| 6.1.3 | Static methods | `static` → `impl` associated functions |
| 6.1.4 | Abstract classes | `abstract` → trait definition |

### Micro 6.2: Advanced Expressions

| Nano | Feature | Description |
|------|---------|-------------|
| 6.2.1 | Spread operator | `...arr` → `.iter().cloned()` |
| 6.2.2 | Rest parameters | `...args: T[]` → `args: Vec<T>` |
| 6.2.3 | Nullish assignment | `??=` → `if option.is_none() { ... }` |
| 6.2.4 | Type narrowing | `typeof x === "string"` → match guards |

### Micro 6.3: Advanced Types

| Nano | Feature | Description |
|------|---------|-------------|
| 6.3.1 | Intersection types | `A & B` → struct composition |
| 6.3.2 | Mapped types | `Partial<T>`, `Required<T>` → Option wrapping |
| 6.3.3 | Index signatures | `[key: string]: T` → HashMap field |
| 6.3.4 | Discriminated unions | `type A = {kind: "a"} \| {kind: "b"}` → enum |

---

## Phase 7: Academic (Milestone 16)

### Micro 7.1: Benchmarks ✅ (Partial — speed benchmarks complete)
- ✅ Criterion.rs benchmark suite (3 groups: full_pipeline by tier, pipeline_stages, scalability)
- ✅ Node.js vs Rust runtime comparison (5 algorithms, avg 35x speedup, semantic equivalence proven)
- ✅ CI integration (GitHub Actions job with artifact upload)
- ✅ Documentation (`docs/BENCHMARKS.md`)
- 📋 Memory profiling (future: valgrind/heaptrack integration)

### Micro 7.2: Formal Specification
- Complete EBNF grammar for Oxidizable Standard
- Formal type mapping specification

### Micro 7.3: Paper/TCC
- Academic paper draft in `papers/`
- Reproducible benchmark results

---

## Execution Order

**Immediate next:** Phase 4, Micro 4.1 (bug fixes) → then 4.2 (unblock analyzer) → then 4.3 (control flow codegen).

Each nano task follows this workflow:
1. Create issue on GitHub
2. Branch from issue (`feat/`, `fix/`)
3. Write equivalence test FIRST (TDD)
4. Implement minimal fix
5. Run full suite + clippy + fmt
6. Commit with `<type>(<scope>): <subject>`
7. Update GRAMMAR.md if new construct
8. PR → merge

---

## Detailed Plans

| Phase | Plan File |
|-------|-----------|
| Phase 1-2 | `docs/superpowers/plans/2026-03-12-full-refactoring-roadmap.md` (COMPLETE) |
| Phase 3 | `docs/superpowers/plans/2026-03-13-milestone-13-semantic-equivalence.md` (COMPLETE) |
| Phase 4 | `docs/superpowers/plans/2026-03-13-milestone-13-bugfix-and-control-flow.md` (COMPLETE) |
| Phase 5 | `docs/superpowers/plans/milestone-14-stdlib-complete.md` (TO CREATE) |
| Phase 5.5 | `docs/superpowers/plans/2026-03-13-cli-ir-analyzer-evolution.md` (COMPLETE — all 20/20 tasks) |
| Phase 6-10 | `docs/superpowers/plans/2026-03-13-nestjs-full-transpilation-roadmap.md` (IN PROGRESS) |
