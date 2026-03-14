# Strict Compiler Rules Plan

## Task Type
- [x] Backend (Rust compiler configuration)

## Current Enforcement (`.cargo/config.toml`)

| Rule | Flag | Enforced? |
|------|------|-----------|
| No `.unwrap()` | `-Wclippy::unwrap_used` | YES — compile error |
| No `.expect()` | `-Wclippy::expect_used` | YES — compile error |
| No `panic!()` | `-Wclippy::panic` | YES — compile error |
| No `todo!()` | `-Wclippy::todo` | YES — compile error |
| No `unimplemented!()` | `-Wclippy::unimplemented` | YES — compile error |
| No unnecessary clones | `-Wclippy::implicit_clone` | YES — compile error |
| No redundant closures | `-Wclippy::redundant_closure_for_method_calls` | YES — compile error |
| No needless pass by value | `-Wclippy::needless_pass_by_value` | YES — compile error |
| All warnings = errors | `-Dwarnings` | YES — compile error |

## Rules Enforced by Convention (not compiler)

| Rule | How Enforced | Status |
|------|-------------|--------|
| Functions < 50 lines | Pre-commit check + code review | ACHIEVED — 0 violations |
| Files < 400 lines | Code review | ACHIEVED — 0 violations |
| `pub(crate)` not `pub` | Code review | Mostly enforced |
| `quote!` only for codegen | Code review | Enforced |
| Max 5 params per function | Code review + context structs | Enforced |
| Max 4 nesting levels | Code review + early returns | Mostly enforced |

## Pre-commit Hooks (active)

```bash
# scripts/pre-commit
cargo fmt -- --check
cargo clippy --workspace -- -D warnings
```

## Semantic Equivalence Rules (ABSOLUTE)

1. ALL test TypeScript MUST be valid and runnable
2. Generated Rust MUST produce IDENTICAL output
3. Use `assert_output_equivalent()` for all feature tests
4. Never use ports 3000-3002 in tests (user's ports)
5. Use ports 3100+ for test servers

## Future: `cognitive_complexity` and `too_many_lines`

These rules cause too many false positives for match dispatchers.
Instead, enforce via convention:
- Zero functions > 50 lines (achieved, maintained by code review)
- If a new function exceeds 50 lines, it MUST be split before merge
