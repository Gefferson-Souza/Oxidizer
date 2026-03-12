# TypeRust Full Refactoring Roadmap

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild TypeRust from scratch — clean slate tests, strict quality enforcement, refactored codegen modules, and incremental feature coverage from basic TypeScript to NestJS.

**Architecture:** Delete all existing tests. Enforce strict Rust quality rules via clippy.toml, `.cargo/config.toml`, and cargo-deny. Refactor monolithic codegen files (func.rs: 1152→7 files, class.rs: 1033→6 files). Rebuild test suite incrementally using unit tests (fast, no compilation), snapshot tests (medium), and compilation tests (slow, batched). Each tier adds TypeScript features progressively.

**Tech Stack:** Rust (quote!/proc-macro2 for codegen), SWC (TypeScript parsing), insta (snapshots), cargo-nextest (fast parallel tests), cargo-deny (dependency auditing)

---

## Current State Analysis

| Metric | Value | Target |
|--------|-------|--------|
| func.rs lines | 1,152 | < 400 |
| class.rs lines | 1,033 | < 400 |
| orchestrator lines | 505 | < 400 |
| `todo!()` calls | 10 (9 func.rs + 1 class.rs) | 0 |
| `.expect()` violations | 2 | 0 |
| `.unwrap()` in lib code | 3 | 0 |
| Dead code files | 2 (tyrus_ast, analyzer/graph) | 0 |
| Test suite time | 3-5 min | < 30s (unit), < 2min (full) |
| Max nesting depth | 13 levels | 4 levels |
| Max function length | 328 lines | 50 lines |
| Function params > 5 | 2 | 0 |

## File Map

### Files to DELETE
```
tests/src/test_snapshots.rs
tests/src/test_build.rs
tests/src/test_compilation.rs
tests/src/test_types.rs
tests/src/test_generics.rs
tests/src/test_nestjs.rs
tests/src/test_regression.rs
tests/src/test_stdlib_exec.rs
tests/src/test_e2e_exec.rs
tests/src/infrastructure/equivalence.rs
tests/src/infrastructure/mod.rs
tests/src/lib.rs
tests/src/snapshots/*.snap
tests/fixtures/                          # All fixtures (rebuilt incrementally)
crates/tyrus_ast/src/lib.rs              # Dead code (OxInterface, OxFunction unused)
crates/tyrus_analyzer/src/graph.rs       # Dead code (duplicate of tyrus_di::graph)
docs/superpowers/plans/2026-03-12-codegen-refactoring.md  # Superseded by this plan
docs/PLAN.md                             # Superseded
docs/PLAN_CI_DEBUG.md                    # Superseded
```

### Files to CREATE (Infrastructure)
```
.cargo/config.toml                       # Default rustflags for strict linting
clippy.toml                              # Custom clippy thresholds
deny.toml                                # cargo-deny config
rustfmt.toml                             # Formatting rules
```

### Files to CREATE (Test Infrastructure)
```
tests/src/lib.rs                         # New test harness entry point
tests/src/helpers.rs                     # Shared test utilities
tests/src/unit/mod.rs                    # Unit test module
tests/src/unit/expr.rs                   # Expression conversion tests
tests/src/unit/stmt.rs                   # Statement conversion tests
tests/src/unit/types.rs                  # Type mapping tests
tests/src/unit/stdlib.rs                 # Stdlib handler tests
tests/src/snapshot/mod.rs                # Snapshot test module
tests/src/snapshot/tier1.rs              # Basic TS snapshots
tests/src/snapshot/tier2.rs              # Intermediate TS snapshots
tests/src/snapshot/tier3.rs              # Advanced TS snapshots
tests/src/snapshot/tier4_nestjs.rs       # NestJS snapshots
tests/src/compilation/mod.rs             # Compilation verification
tests/src/compilation/tier1.rs           # Basic compilation checks
tests/src/compilation/tier2.rs           # Intermediate compilation checks
tests/src/compilation/nestjs.rs          # NestJS full project compilation
```

### Files to CREATE (Codegen Refactoring)
```
crates/tyrus_codegen/src/convert/helpers.rs        # to_snake_case, to_pascal_case, is_string_expr, is_primitive_type
crates/tyrus_codegen/src/convert/stmt.rs           # convert_stmt, convert_stmt_recursive
crates/tyrus_codegen/src/convert/fn_decl.rs        # process_fn_decl
crates/tyrus_codegen/src/convert/expr/mod.rs       # convert_expr dispatcher
crates/tyrus_codegen/src/convert/expr/binary.rs    # convert_bin_expr
crates/tyrus_codegen/src/convert/expr/call.rs      # convert_call_expr (split into sub-functions)
crates/tyrus_codegen/src/convert/expr/member.rs    # convert_member_expr
crates/tyrus_codegen/src/convert/expr/arrow.rs     # convert_arrow_expr
crates/tyrus_codegen/src/convert/expr/literal.rs   # convert_object_lit, convert_array_lit, convert_tpl
crates/tyrus_codegen/src/convert/expr/misc.rs      # convert_assign_expr, convert_update_expr, convert_opt_chain
crates/tyrus_codegen/src/convert/class/mod.rs      # process_class_decl (dispatcher only)
crates/tyrus_codegen/src/convert/class/struct_gen.rs   # Struct definition generation
crates/tyrus_codegen/src/convert/class/constructor.rs  # Constructor transpilation
crates/tyrus_codegen/src/convert/class/method.rs       # Method transpilation
crates/tyrus_codegen/src/convert/class/routing.rs      # HTTP route generation (Axum)
crates/tyrus_codegen/src/convert/class/mutation.rs     # Self-mutation detection
crates/tyrus_orchestrator/src/pipeline.rs          # build_project core logic
crates/tyrus_orchestrator/src/scaffold.rs           # generate_main_rs, generate_cargo_toml, generate_mod_rs
crates/tyrus_orchestrator/src/format.rs             # format_code, get_app_error_code
```

### Files to MODIFY
```
crates/tyrus_codegen/src/convert/mod.rs            # Add new module declarations
crates/tyrus_codegen/src/convert/func.rs           # Reduce to thin re-export layer → DELETE after migration
crates/tyrus_codegen/src/convert/class.rs          # Reduce to thin re-export layer → DELETE after migration
crates/tyrus_codegen/src/convert/type_mapper.rs    # Deduplicate map_ts_type / map_inner_type
crates/tyrus_orchestrator/src/lib.rs               # Extract to pipeline/scaffold/format modules
crates/tyrus_common/src/util.rs                    # Fix unwrap_or violation
crates/tyrus_analyzer/src/lib.rs                   # Remove graph module reference
tests/Cargo.toml                                   # Update dependencies
.github/workflows/ci.yml                           # Add cargo-deny, nextest
CLAUDE.md                                          # Update architecture section
```

### Fixture Files to CREATE (Incrementally, per Tier)
```
tests/fixtures/tier1/                              # Basic TS
  variables.ts                                     # const, let declarations
  math_ops.ts                                      # +, -, *, /, %
  string_ops.ts                                    # Template literals, concatenation
  functions.ts                                     # Sync functions, params, return types
  control_flow.ts                                  # if/else, while
  console.ts                                       # console.log, console.error

tests/fixtures/tier2/                              # Intermediate TS
  interfaces.ts                                    # interface → struct
  type_aliases.ts                                  # type alias, string unions → enum
  arrays.ts                                        # Array creation, map, filter, forEach
  classes.ts                                       # class, constructor, methods
  async_await.ts                                   # async/await, Promise<T>

tests/fixtures/tier3/                              # Advanced TS
  generics.ts                                      # Generic interface, class, function
  optional_chaining.ts                             # ?., ??
  destructuring.ts                                 # Object/array destructuring
  advanced_methods.ts                              # .find, .some, .every, .reduce
  string_methods.ts                                # .includes, .replace, .split, etc.
  math_stdlib.ts                                   # Math.max, Math.min, Math.round, etc.

tests/fixtures/tier4/                              # NestJS
  injectable_service.ts                            # @Injectable, DI
  controller.ts                                    # @Controller, @Get, @Post
  full_project/                                    # Multi-file NestJS project
    src/app.module.ts
    src/users/users.module.ts
    src/users/users.service.ts
    src/users/users.controller.ts
    src/users/dto/create-user.dto.ts
```

---

## Chunk 1: Clean Slate & Strict Rules

### Task 1: Delete All Existing Tests and Dead Code

**Files:**
- Delete: `tests/src/*.rs`, `tests/src/infrastructure/`, `tests/src/snapshots/`
- Delete: All `tests/fixtures/*/` directories
- Delete: `crates/tyrus_ast/src/lib.rs` content (keep crate shell)
- Delete: `crates/tyrus_analyzer/src/graph.rs`
- Delete: `docs/PLAN.md`, `docs/PLAN_CI_DEBUG.md`, `docs/superpowers/plans/2026-03-12-codegen-refactoring.md`
- Modify: `crates/tyrus_analyzer/src/lib.rs` (remove graph module)

- [ ] **Step 1: Delete all test source files**

```bash
rm -rf tests/src/test_*.rs
rm -rf tests/src/infrastructure/
rm -rf tests/src/snapshots/
rm -rf tests/fixtures/
```

- [ ] **Step 2: Create minimal test harness**

Create `tests/src/lib.rs`:
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod helpers;
mod unit;
mod snapshot;
mod compilation;
```

Create `tests/src/helpers.rs`:
```rust
use std::path::Path;
use tyrus_orchestrator;
use tyrus_parser;

/// Transpile a TypeScript string to Rust code (no compilation).
/// This is the primary test helper — fast, no I/O.
pub fn transpile(ts_code: &str) -> String {
    let tmp = tempfile::NamedTempFile::with_suffix(".ts").expect("tmp file");
    std::fs::write(tmp.path(), ts_code).expect("write ts");
    tyrus_orchestrator::build(tmp.path().into())
        .unwrap_or_else(|e| panic!("Transpilation failed: {e}"))
}

/// Transpile a fixture file by name (e.g., "tier1/variables").
pub fn transpile_fixture(fixture: &str) -> String {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(format!("{fixture}.ts"));
    assert!(fixture_path.exists(), "Fixture not found: {}", fixture_path.display());
    tyrus_orchestrator::build(fixture_path.into())
        .unwrap_or_else(|e| panic!("Transpilation failed for {fixture}: {e}"))
}

/// Parse TypeScript and return the SWC AST (for unit-testing codegen functions directly).
pub fn parse_ts(ts_code: &str) -> swc_ecma_ast::Program {
    let tmp = tempfile::NamedTempFile::with_suffix(".ts").expect("tmp file");
    std::fs::write(tmp.path(), ts_code).expect("write ts");
    tyrus_parser::parse(tmp.path())
        .unwrap_or_else(|e| panic!("Parse failed: {e}"))
}
```

Create `tests/src/unit/mod.rs`:
```rust
mod expr;
mod stmt;
mod types;
mod stdlib;
```

Create `tests/src/snapshot/mod.rs`:
```rust
mod tier1;
```

Create `tests/src/compilation/mod.rs`:
```rust
mod tier1;
```

Create placeholder files (empty modules for now):
```rust
// tests/src/unit/expr.rs
// tests/src/unit/stmt.rs
// tests/src/unit/types.rs
// tests/src/unit/stdlib.rs
// tests/src/snapshot/tier1.rs
// tests/src/compilation/tier1.rs
```

- [ ] **Step 3: Delete dead code**

Remove `crates/tyrus_ast/src/lib.rs` content (keep empty crate):
```rust
// tyrus_ast: Reserved for future typed IR definitions.
// Currently empty — SWC AST is used directly.
```

Remove `crates/tyrus_analyzer/src/graph.rs` and update `lib.rs`:
```rust
// crates/tyrus_analyzer/src/lib.rs
// Remove: mod graph;
// Keep: mod lints; mod decorators;
pub mod decorators;
pub mod lints;

use tyrus_di::graph::DiGraph;
// ... rest unchanged
```

Delete obsolete docs:
```bash
rm docs/PLAN.md docs/PLAN_CI_DEBUG.md
rm docs/superpowers/plans/2026-03-12-codegen-refactoring.md
```

- [ ] **Step 4: Verify project still builds**

```bash
cargo build --workspace
```
Expected: SUCCESS (no functional code was removed, only tests and dead code)

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "chore: clean slate — delete all tests, dead code, and obsolete docs"
```

---

### Task 2: Set Up Strict Project Rules

**Files:**
- Create: `.cargo/config.toml`
- Create: `clippy.toml`
- Create: `deny.toml`
- Create: `rustfmt.toml`
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Create `.cargo/config.toml` for strict linting**

> **IMPORTANT:** Do NOT add `-Dwarnings` yet — existing violations must be fixed first (Task 3).
> After Task 3 is complete, Step 1b below promotes warnings to errors.

```toml
# .cargo/config.toml
# These flags apply to ALL cargo commands in this workspace.
# Step 1: Warnings only (will be promoted to errors after violations are fixed in Task 3).

[target.'cfg(all())']
rustflags = [
    # Forbid unsafe error handling (enforced by clippy)
    "-Wclippy::unwrap_used",
    "-Wclippy::expect_used",
    "-Wclippy::panic",
    # Forbid incomplete implementations
    "-Wclippy::todo",
    "-Wclippy::unimplemented",
    # Code quality
    "-Wclippy::needless_pass_by_value",
    "-Wclippy::redundant_closure_for_method_calls",
    "-Wclippy::manual_let_else",
    "-Wclippy::implicit_clone",
]
```

- [ ] **Step 2: Create `clippy.toml` for structural quality thresholds**

```toml
# clippy.toml
# Structural quality thresholds — violations are warnings (promoted to errors by -Dwarnings).

# Functions with more than 5 parameters need a context struct
max-fn-params = 5

# Cognitive complexity threshold — keep functions simple
cognitive-complexity-threshold = 15

# Maximum lines per function — extract helpers beyond this
too-many-lines-threshold = 50

# Type complexity threshold
type-complexity-threshold = 250
```

- [ ] **Step 3: Create `rustfmt.toml` for consistent formatting**

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
```

- [ ] **Step 4: Create `deny.toml` for dependency auditing**

```toml
# deny.toml — cargo-deny configuration
# Run: cargo deny check

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "OpenSSL",
    "Zlib",
    "BSL-1.0",
    "CC0-1.0",
    "MPL-2.0",
]

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

- [ ] **Step 5: Update CI pipeline**

Modify `.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: ["main", "feat/*", "chore/*", "refactor/*", "fix/*"]
  pull_request:
    branches: ["main"]

env:
  CARGO_TERM_COLOR: always
  CARGO_INCREMENTAL: 0
  RUSTFLAGS: "-Dwarnings"

jobs:
  quality:
    name: Quality Checks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2

      - name: Check Formatting
        run: cargo fmt -- --check

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Dependency Audit
        uses: EmbarkStudios/cargo-deny-action@v1

      - name: Build
        run: cargo build --workspace

  test:
    name: Tests
    runs-on: ubuntu-latest
    needs: quality
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install nextest
        uses: taiki-e/install-action@nextest

      - name: Unit & Snapshot Tests
        run: cargo nextest run --workspace

      - name: Verify Real-World Demo
        run: |
          cargo build --bin tyrus
          ./target/debug/tyrus build examples/real_world_demo/src --output examples/real_world_demo/output
          cd examples/real_world_demo/output && cargo check

  release:
    name: Release
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    needs: [quality, test]
    permissions:
      contents: write
      pull-requests: write
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@stable
      - uses: MarcoIeni/release-plz-action@v0.5
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

- [ ] **Step 6: Verify rules are active (warnings, not errors yet)**

```bash
cargo clippy --workspace 2>&1 | grep "warning:"
```
Expected: Warnings for existing violations (todo!, expect, etc.) — proves rules are active. Build still succeeds.

- [ ] **Step 7: Commit**

```bash
git add .cargo/config.toml clippy.toml deny.toml rustfmt.toml .github/workflows/ci.yml
git commit -m "chore: add strict project rules — clippy.toml, deny.toml, cargo config, CI pipeline"
```

---

### Task 3: Fix All Existing Code Violations

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/func.rs` (replace 9 `todo!()`)
- Modify: `crates/tyrus_codegen/src/convert/class.rs` (replace 1 `todo!()`, fix clippy)
- Modify: `crates/tyrus_orchestrator/src/lib.rs` (replace 2 `.expect()`)
- Modify: `crates/tyrus_common/src/util.rs` (fix `.unwrap_or()`)
- Note: `crates/tyrus_codegen/src/stdlib/json.rs` uses `.unwrap_or_default()` in GENERATED code — this is acceptable (not library code). No fix needed.

- [ ] **Step 1: Replace all `todo!()` in func.rs with proper error tokens**

In `crates/tyrus_codegen/src/convert/func.rs`, replace every `todo!(...)` with a compile-error token that surfaces clearly in generated code:

```rust
// Replace this pattern everywhere:
//   todo!("unsupported literal")
// With:
//   quote! { compile_error!("Tyrus: unsupported literal type") }

// Line 419 (unknown statement):
_ => quote! { /* Tyrus: unsupported statement */ },

// Line 455 (unsupported literal):
_ => quote! { compile_error!("Tyrus: unsupported literal type") },

// Line 489 (unknown expression):
_ => quote! { compile_error!("Tyrus: unsupported expression") },

// Line 639 (unhandled member expression):
_ => {
    let obj_tokens = self.convert_expr(obj);
    let prop_str = format!("{prop:?}");
    let prop_ident = format_ident!("{}", prop_str);
    quote! { #obj_tokens.#prop_ident }
}

// Line 669 (unsupported binary op):
_ => {
    let left = self.convert_expr(&bin.left);
    let right = self.convert_expr(&bin.right);
    quote! { compile_error!("Tyrus: unsupported binary operator") }
}

// Line 1012 (complex callee):
_ => quote! { compile_error!("Tyrus: unsupported call expression") },

// Lines 1087, 1095, 1102, 1105 (complex assignments):
_ => quote! { compile_error!("Tyrus: unsupported assignment pattern") },
```

- [ ] **Step 2: Fix class.rs violations**

In `crates/tyrus_codegen/src/convert/class.rs`:

```rust
// Line 778: Replace todo!() with error token
// OLD: todo!("Complex constructor not yet supported")
// NEW:
_ => quote! { compile_error!("Tyrus: complex constructor pattern not yet supported") },

// Line 834: Fix clippy::unnecessary_map_or
// OLD: .map_or(false, |body| Self::body_mutates_self(&body.stmts))
// NEW:
.is_some_and(|body| Self::body_mutates_self(&body.stmts))

// Line 990: Fix clippy::unnecessary_map_or
// OLD: .map_or(false, |alt| ...)
// NEW:
.is_some_and(|alt| ...)
```

- [ ] **Step 3: Fix orchestrator `.expect()` violations**

In `crates/tyrus_orchestrator/src/lib.rs`:

```rust
// Lines 325, 329: Replace .expect() in generated main.rs code
// The generated code uses .expect() — change to proper error handling in generated code.

// OLD (in the format! string for main.rs):
//   .expect("Failed to bind to address")
//   .expect("Server failed")
// NEW:
//   .map_err(|e| eprintln!("Failed to bind: {e}")).unwrap_or_else(|_| std::process::exit(1))
// OR better — generate main() -> Result<(), Box<dyn std::error::Error>>:

// Change the generated main function signature to return Result:
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     ...
//     let listener = TcpListener::bind("0.0.0.0:3000").await?;
//     axum::serve(listener, app).await?;
//     Ok(())
// }
```

- [ ] **Step 4: Fix tyrus_common util.rs**

In `crates/tyrus_common/src/util.rs`:

```rust
// Line 7: Replace .unwrap_or(ch) with .extend()
// OLD: result.push(ch.to_lowercase().next().unwrap_or(ch));
// NEW:
result.extend(ch.to_lowercase());
```

- [ ] **Step 5: Run cargo fmt**

```bash
cargo fmt --all
```

- [ ] **Step 6: Verify all violations are fixed**

```bash
cargo clippy --workspace
```
Expected: SUCCESS (no warnings or errors)

- [ ] **Step 7: Verify build passes**

```bash
cargo build --workspace
```
Expected: SUCCESS

- [ ] **Step 8: Promote warnings to errors in `.cargo/config.toml`**

Add `-Dwarnings` to the rustflags array now that all violations are fixed:
```toml
[target.'cfg(all())']
rustflags = [
    # NOW promoted to errors — all violations fixed
    "-Dwarnings",
    "-Wclippy::unwrap_used",
    # ... rest unchanged
]
```

- [ ] **Step 9: Verify strict mode passes**

```bash
cargo clippy --workspace
```
Expected: SUCCESS (zero warnings = zero errors)

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "fix: resolve all clippy violations and promote warnings to errors"
```

---

## Chunk 2: Codegen Refactoring — func.rs Decomposition

### Task 4: Extract helpers.rs

**Files:**
- Create: `crates/tyrus_codegen/src/convert/helpers.rs`
- Modify: `crates/tyrus_codegen/src/convert/func.rs` (remove helpers)
- Modify: `crates/tyrus_codegen/src/convert/mod.rs` (add module)

- [ ] **Step 1: Write tests for helpers**

Create `tests/src/unit/expr.rs` (initial helper tests):
```rust
use crate::helpers::parse_ts;

#[test]
fn test_to_snake_case_simple() {
    assert_eq!(tyrus_common::util::to_snake_case("getUserName"), "get_user_name");
}

#[test]
fn test_to_snake_case_acronym() {
    assert_eq!(tyrus_common::util::to_snake_case("XMLParser"), "xmlparser");
}

#[test]
fn test_to_snake_case_already_snake() {
    assert_eq!(tyrus_common::util::to_snake_case("already_snake"), "already_snake");
}
```

- [ ] **Step 2: Run tests to verify they pass**

```bash
cargo test -p integration_tests unit::expr
```
Expected: PASS (these test existing functionality)

- [ ] **Step 3: Create helpers.rs**

Create `crates/tyrus_codegen/src/convert/helpers.rs`:
```rust
//! Shared helper functions for code generation.
//!
//! Contains utility functions used across multiple codegen modules:
//! - Case conversion (to_snake_case, to_pascal_case)
//! - Expression type detection (is_string_expr, is_primitive_type)
//! - Mutex access pattern generation

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

/// List of Rust primitive type names used to detect dependencies vs value types.
pub(crate) const PRIMITIVE_TYPES: &[&str] = &[
    "String", "f64", "bool", "i32", "i64", "u32", "u64", "usize",
    "Vec", "Option", "HashMap", "Array",
];

/// Check if a type name is a Rust primitive/stdlib type (not a dependency).
pub(crate) fn is_primitive_type(name: &str) -> bool {
    PRIMITIVE_TYPES.contains(&name)
}

/// Convert camelCase or PascalCase to snake_case.
/// NOTE: This is the func.rs version (handles consecutive uppercase correctly).
/// The tyrus_common::util version differs — do NOT use it for codegen.
/// Example: "getUserName" → "get_user_name", "HTTPRequest" → "httprequest"
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut was_upper = false;
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !was_upper {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            was_upper = true;
        } else {
            result.push(c);
            was_upper = false;
        }
    }
    result
}

/// Convert snake_case, camelCase, or kebab-case to PascalCase.
/// Handles both `_` and `-` delimiters.
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut next_upper = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            next_upper = true;
        } else if next_upper {
            result.push(c.to_ascii_uppercase());
            next_upper = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Heuristic: detect if an expression is likely string-typed.
/// Used to decide between `format!("{}{}", a, b)` (string concat)
/// and `a + b` (numeric addition).
pub(crate) fn is_string_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(Lit::Str(_)) => true,
        Expr::Tpl(_) => true,
        Expr::Call(call) => {
            if let Callee::Expr(callee_expr) = &call.callee {
                if let Expr::Member(member) = &**callee_expr {
                    if let MemberProp::Ident(method) = &member.prop {
                        let method_name = method.sym.as_ref();
                        return matches!(
                            method_name,
                            "toString"
                                | "toUpperCase"
                                | "toLowerCase"
                                | "trim"
                                | "replace"
                                | "slice"
                                | "substring"
                        );
                    }
                }
                // String() constructor
                if let Expr::Ident(ident) = &**callee_expr {
                    return ident.sym.as_ref() == "String";
                }
            }
            false
        }
        Expr::Bin(bin) if bin.op == BinaryOp::Add => {
            is_string_expr(&bin.left) || is_string_expr(&bin.right)
        }
        _ => false,
    }
}

/// Generate a safe mutex lock access pattern.
/// Produces: `self.{field}.lock().unwrap_or_else(|e| e.into_inner())`
pub(crate) fn gen_mutex_lock(field: &proc_macro2::Ident) -> TokenStream {
    quote! { self.#field.lock().unwrap_or_else(|e| e.into_inner()) }
}
```

- [ ] **Step 4: Update mod.rs to include helpers**

In `crates/tyrus_codegen/src/convert/mod.rs`, add:
```rust
pub(crate) mod helpers;
```

- [ ] **Step 5: Update func.rs to use helpers**

In `crates/tyrus_codegen/src/convert/func.rs`:
- Remove `to_snake_case` function (lines 10-26)
- Remove `to_pascal_case` function (lines 29-43)
- Remove `is_string_expr` function (lines 676-714)
- Add import: `use super::helpers::{to_snake_case, to_pascal_case, is_string_expr, is_primitive_type};`
- **CRITICAL:** Update ALL call sites in func.rs:
  - `Self::is_string_expr(...)` → `is_string_expr(...)` (in `convert_bin_expr`, around line 650)
  - Any other `Self::` references to moved functions

- [ ] **Step 6: Update class.rs to use helpers**

In `crates/tyrus_codegen/src/convert/class.rs`:
- Replace all inline `"String" | "f64" | "bool" | ...` matches with `is_primitive_type(name)`
- Add import: `use super::helpers::is_primitive_type;`

- [ ] **Step 7: Verify build and existing behavior**

```bash
cargo build --workspace && cargo clippy --workspace
```
Expected: SUCCESS

- [ ] **Step 8: Commit**

```bash
git add crates/tyrus_codegen/src/convert/helpers.rs crates/tyrus_codegen/src/convert/mod.rs crates/tyrus_codegen/src/convert/func.rs crates/tyrus_codegen/src/convert/class.rs
git commit -m "refactor: extract helpers.rs from func.rs — shared utilities for codegen"
```

---

### Task 5: Extract stmt.rs from func.rs

**Files:**
- Create: `crates/tyrus_codegen/src/convert/stmt.rs`
- Modify: `crates/tyrus_codegen/src/convert/func.rs`
- Modify: `crates/tyrus_codegen/src/convert/mod.rs`

- [ ] **Step 1: Create stmt.rs with statement conversion logic**

Move `convert_stmt` (lines 217-421) and `convert_stmt_recursive` (lines 172-215) from func.rs to a new `crates/tyrus_codegen/src/convert/stmt.rs`:

```rust
//! Statement-level code generation.
//!
//! Converts TypeScript statements (variable declarations, if/else, while, for-of,
//! return, throw, try-catch, switch, do-while) to Rust TokenStreams.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use super::helpers::to_snake_case;
use super::interface::RustGenerator;

impl RustGenerator {
    /// Recursively convert statements with a custom return handler.
    /// Used by process_fn_decl to inject Result wrapping for async functions.
    pub fn convert_stmt_recursive<F>(
        &self,
        stmt: &Stmt,
        return_handler: &mut F,
    ) -> TokenStream
    where
        F: FnMut(&swc_ecma_ast::ReturnStmt) -> TokenStream,
    {
        // ... move existing code from func.rs lines 172-215
    }

    /// Convert a single TypeScript statement to Rust tokens.
    pub fn convert_stmt(&self, stmt: &Stmt) -> TokenStream {
        // ... move existing code from func.rs lines 217-421
    }
}
```

- [ ] **Step 2: Update mod.rs**

```rust
pub(crate) mod stmt;
```

- [ ] **Step 3: Remove statement code from func.rs**

Remove `convert_stmt` and `convert_stmt_recursive` from func.rs. Add `use` if needed.

- [ ] **Step 4: Verify build**

```bash
cargo build --workspace && cargo clippy --workspace
```
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add crates/tyrus_codegen/src/convert/stmt.rs crates/tyrus_codegen/src/convert/func.rs crates/tyrus_codegen/src/convert/mod.rs
git commit -m "refactor: extract stmt.rs — statement conversion separated from func.rs"
```

---

### Task 6: Extract fn_decl.rs from func.rs

**Files:**
- Create: `crates/tyrus_codegen/src/convert/fn_decl.rs`
- Modify: `crates/tyrus_codegen/src/convert/func.rs`
- Modify: `crates/tyrus_codegen/src/convert/mod.rs`

- [ ] **Step 1: Create fn_decl.rs**

Move `process_fn_decl` (lines 46-169) from func.rs to `crates/tyrus_codegen/src/convert/fn_decl.rs`:

```rust
//! Function declaration transpilation.
//!
//! Converts TypeScript function declarations to Rust function items.
//! Handles: sync/async, return types, parameter mapping, Result wrapping.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use super::helpers::to_snake_case;
use super::interface::RustGenerator;
use super::type_mapper;

impl RustGenerator {
    /// Convert a TypeScript function declaration to a Rust function.
    pub fn process_fn_decl(&mut self, n: &FnDecl) {
        // ... move existing code from func.rs lines 46-169
    }
}
```

- [ ] **Step 2: Update mod.rs**

```rust
pub(crate) mod fn_decl;
```

- [ ] **Step 3: Remove from func.rs and verify**

```bash
cargo build --workspace && cargo clippy --workspace
```

- [ ] **Step 4: Commit**

```bash
git add crates/tyrus_codegen/src/convert/fn_decl.rs crates/tyrus_codegen/src/convert/func.rs crates/tyrus_codegen/src/convert/mod.rs
git commit -m "refactor: extract fn_decl.rs — function declaration processing separated"
```

---

### Task 7: Create expr/ Module from func.rs

**Files:**
- Create: `crates/tyrus_codegen/src/convert/expr/mod.rs`
- Create: `crates/tyrus_codegen/src/convert/expr/binary.rs`
- Create: `crates/tyrus_codegen/src/convert/expr/call.rs`
- Create: `crates/tyrus_codegen/src/convert/expr/member.rs`
- Create: `crates/tyrus_codegen/src/convert/expr/arrow.rs`
- Create: `crates/tyrus_codegen/src/convert/expr/literal.rs`
- Create: `crates/tyrus_codegen/src/convert/expr/misc.rs`
- Modify: `crates/tyrus_codegen/src/convert/mod.rs`
- Delete: `crates/tyrus_codegen/src/convert/func.rs` (after full migration)

- [ ] **Step 1: Create expr/mod.rs — the expression dispatcher**

```rust
//! Expression-level code generation.
//!
//! Dispatches TypeScript expressions to specialized handlers.

mod binary;
mod call;
mod member;
mod arrow;
mod literal;
mod misc;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use super::helpers::{to_snake_case, is_string_expr};
use super::interface::RustGenerator;

impl RustGenerator {
    /// Main expression dispatcher — routes each expression type to its handler.
    pub fn convert_expr(&self, expr: &Expr) -> TokenStream {
        match expr {
            Expr::Lit(lit) => self.convert_lit(lit),
            Expr::Ident(ident) => {
                let name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                quote! { #name }
            }
            Expr::Bin(bin) => self.convert_bin_expr(bin),
            Expr::Call(call) => self.convert_call_expr(call),
            Expr::Member(member) => self.convert_member_expr(member),
            Expr::Arrow(arrow) => self.convert_arrow_expr(arrow),
            Expr::Object(obj) => self.convert_object_lit(obj),
            Expr::Array(arr) => self.convert_array_lit(arr),
            Expr::Tpl(tpl) => self.convert_tpl(tpl),
            Expr::Assign(assign) => self.convert_assign_expr(assign),
            Expr::Update(update) => self.convert_update_expr(update),
            Expr::Paren(paren) => self.convert_expr(&paren.expr),
            Expr::Unary(unary) => self.convert_unary_expr(unary),
            Expr::Await(await_expr) => {
                let inner = self.convert_expr(&await_expr.arg);
                quote! { #inner.await? }
            }
            Expr::New(new_expr) => self.convert_new_expr(new_expr),
            Expr::Cond(cond) => self.convert_cond_expr(cond),
            Expr::OptChain(opt) => self.convert_opt_chain(opt),
            Expr::This(_) => quote! { self },
            _ => quote! { compile_error!("Tyrus: unsupported expression") },
        }
    }

    pub fn convert_expr_or_spread(&self, arg: &ExprOrSpread) -> TokenStream {
        self.convert_expr(&arg.expr)
    }

    fn convert_lit(&self, lit: &Lit) -> TokenStream {
        match lit {
            Lit::Num(n) => {
                let val = n.value;
                quote! { #val }
            }
            Lit::Str(s) => {
                let val = s.value.as_ref();
                quote! { #val.to_string() }
            }
            Lit::Bool(b) => {
                let val = b.value;
                quote! { #val }
            }
            Lit::Null(_) => quote! { None },
            _ => quote! { compile_error!("Tyrus: unsupported literal type") },
        }
    }

    fn convert_unary_expr(&self, unary: &UnaryExpr) -> TokenStream {
        let arg = self.convert_expr(&unary.arg);
        match unary.op {
            UnaryOp::Minus => quote! { -#arg },
            UnaryOp::Plus => quote! { #arg },
            UnaryOp::Bang => quote! { !#arg },
            UnaryOp::TypeOf => quote! { std::any::type_name_of_val(&#arg) },
            _ => quote! { compile_error!("Tyrus: unsupported unary operator") },
        }
    }

    fn convert_new_expr(&self, new_expr: &NewExpr) -> TokenStream {
        if let Expr::Ident(ident) = &*new_expr.callee {
            let class_name = format_ident!("{}", ident.sym.as_ref());
            let args: Vec<TokenStream> = new_expr
                .args
                .as_ref()
                .map(|a| a.iter().map(|arg| self.convert_expr(&arg.expr)).collect())
                .unwrap_or_default();
            quote! { #class_name::new(#(#args),*) }
        } else {
            quote! { compile_error!("Tyrus: unsupported new expression") }
        }
    }

    fn convert_cond_expr(&self, cond: &CondExpr) -> TokenStream {
        let test = self.convert_expr(&cond.test);
        let cons = self.convert_expr(&cond.cons);
        let alt = self.convert_expr(&cond.alt);
        quote! { if #test { #cons } else { #alt } }
    }
}
```

- [ ] **Step 2: Create expr/binary.rs**

```rust
//! Binary expression code generation.
//!
//! Maps TypeScript binary operators to Rust equivalents.
//! Special handling: string concatenation (format!) vs numeric addition (+).

use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use crate::convert::helpers::is_string_expr;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    pub(crate) fn convert_bin_expr(&self, bin: &BinExpr) -> TokenStream {
        let left = self.convert_expr(&bin.left);
        let right = self.convert_expr(&bin.right);

        match bin.op {
            BinaryOp::Add => {
                if is_string_expr(&bin.left) || is_string_expr(&bin.right) {
                    quote! { format!("{}{}", #left, #right) }
                } else {
                    quote! { #left + #right }
                }
            }
            BinaryOp::Sub => quote! { #left - #right },
            BinaryOp::Mul => quote! { #left * #right },
            BinaryOp::Div => quote! { #left / #right },
            BinaryOp::Mod => quote! { #left % #right },
            BinaryOp::EqEqEq | BinaryOp::EqEq => quote! { #left == #right },
            BinaryOp::NotEqEq | BinaryOp::NotEq => quote! { #left != #right },
            BinaryOp::Lt => quote! { #left < #right },
            BinaryOp::LtEq => quote! { #left <= #right },
            BinaryOp::Gt => quote! { #left > #right },
            BinaryOp::GtEq => quote! { #left >= #right },
            BinaryOp::LogicalAnd => quote! { #left && #right },
            BinaryOp::LogicalOr => quote! { #left || #right },
            BinaryOp::NullishCoalescing => quote! { #left.unwrap_or(#right) },
            _ => quote! { compile_error!("Tyrus: unsupported binary operator") },
        }
    }
}
```

- [ ] **Step 3: Create expr/call.rs, expr/member.rs, expr/arrow.rs, expr/literal.rs, expr/misc.rs**

Each file follows the same pattern — extract the corresponding method from func.rs into its own file. Keep each under 200 lines.

**expr/call.rs**: `convert_call_expr` — split the 305-line method into:
  - `convert_call_expr` (dispatcher, ~50 lines)
  - `convert_stdlib_call` (console, Math, JSON — delegates to stdlib/)
  - `convert_array_method_call` (map, filter, forEach, find, etc.)
  - `convert_fetch_call` (fetch/axios → reqwest)

**expr/member.rs**: `convert_member_expr` — member access, mutex state fields

**expr/arrow.rs**: `convert_arrow_expr` — arrow function conversion

**expr/literal.rs**: `convert_object_lit`, `convert_array_lit`, `convert_tpl`

**expr/misc.rs**: `convert_assign_expr`, `convert_update_expr`, `convert_opt_chain`

- [ ] **Step 4: Delete func.rs (after verifying all code migrated)**

```bash
rm crates/tyrus_codegen/src/convert/func.rs
```

- [ ] **Step 5: Update mod.rs**

```rust
// crates/tyrus_codegen/src/convert/mod.rs
pub mod interface;
pub(crate) mod helpers;
pub(crate) mod stmt;
pub(crate) mod fn_decl;
pub(crate) mod expr;
pub mod class;  // will be split later
pub mod type_mapper;
```

- [ ] **Step 6: Verify build and line counts**

```bash
cargo build --workspace && cargo clippy --workspace
wc -l crates/tyrus_codegen/src/convert/expr/*.rs
wc -l crates/tyrus_codegen/src/convert/*.rs
```
Expected: Each file < 400 lines, total functionality preserved.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor: decompose func.rs into expr/ module — 1152 lines → 7 focused files"
```

---

### Task 8: Deduplicate type_mapper.rs

**Files:**
- Modify: `crates/tyrus_codegen/src/convert/type_mapper.rs`

- [ ] **Step 1: Write test for type mapping**

In `tests/src/unit/types.rs`:
```rust
use crate::helpers::transpile;

#[test]
fn test_type_mapping_string() {
    let rust = transpile("const name: string = \"hello\";");
    assert!(rust.contains("String"), "string should map to String");
}

#[test]
fn test_type_mapping_number() {
    let rust = transpile("const x: number = 42;");
    assert!(rust.contains("f64"), "number should map to f64");
}

#[test]
fn test_type_mapping_boolean() {
    let rust = transpile("const flag: boolean = true;");
    assert!(rust.contains("bool"), "boolean should map to bool");
}

#[test]
fn test_type_mapping_array() {
    let rust = transpile("const nums: number[] = [1, 2, 3];");
    assert!(rust.contains("Vec<f64>"), "number[] should map to Vec<f64>");
}
```

- [ ] **Step 2: Consolidate map_ts_type and map_inner_type**

In `crates/tyrus_codegen/src/convert/type_mapper.rs`, merge the two ~80% identical functions:

```rust
/// Core type mapping — converts a TsType node to Rust TokenStream.
/// This is the single source of truth for all TS→Rust type conversions.
fn map_type_core(ts_type: &TsType) -> TokenStream {
    match ts_type {
        TsType::TsKeywordType(kw) => match kw.kind {
            TsKeywordTypeKind::TsStringKeyword => quote! { String },
            TsKeywordTypeKind::TsNumberKeyword => quote! { f64 },
            TsKeywordTypeKind::TsBooleanKeyword => quote! { bool },
            TsKeywordTypeKind::TsVoidKeyword => quote! { () },
            TsKeywordTypeKind::TsNullKeyword => quote! { () },
            TsKeywordTypeKind::TsUndefinedKeyword => quote! { () },
            TsKeywordTypeKind::TsAnyKeyword => quote! { serde_json::Value },
            _ => quote! { serde_json::Value },
        },
        TsType::TsArrayType(arr) => {
            let inner = map_type_core(&arr.elem_type);
            quote! { Vec<#inner> }
        }
        TsType::TsTypeRef(type_ref) => {
            // Handle Promise<T>, Array<T>, Record<K,V>, etc.
            // ... consolidated logic
        }
        // ... other type patterns
        _ => quote! { serde_json::Value },
    }
}

/// Public API: map from type annotation (Option<&Box<TsTypeAnn>>)
pub fn map_ts_type(type_ann: Option<&Box<TsTypeAnn>>) -> TokenStream {
    type_ann
        .map(|ann| map_type_core(&ann.type_ann))
        .unwrap_or_else(|| quote! { serde_json::Value })
}

/// Public API: map from TsType reference directly
pub fn map_inner_type(ts_type: &TsType) -> TokenStream {
    map_type_core(ts_type)
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p integration_tests unit::types
```
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add crates/tyrus_codegen/src/convert/type_mapper.rs tests/src/unit/types.rs
git commit -m "refactor: deduplicate type_mapper — consolidate map_ts_type and map_inner_type"
```

---

## Chunk 3: Codegen Refactoring — class.rs & Orchestrator

### Task 9: Split class.rs into class/ Module

**Files:**
- Create: `crates/tyrus_codegen/src/convert/class/mod.rs`
- Create: `crates/tyrus_codegen/src/convert/class/struct_gen.rs`
- Create: `crates/tyrus_codegen/src/convert/class/constructor.rs`
- Create: `crates/tyrus_codegen/src/convert/class/method.rs`
- Create: `crates/tyrus_codegen/src/convert/class/routing.rs`
- Create: `crates/tyrus_codegen/src/convert/class/mutation.rs`
- Delete: `crates/tyrus_codegen/src/convert/class.rs`

- [ ] **Step 1: Create class/mod.rs as the dispatcher**

The monolithic `process_class_decl` (1033 lines) becomes a ~150-line orchestrator that delegates to sub-modules.

```rust
//! Class transpilation module.
//!
//! Converts TypeScript classes to Rust struct + impl blocks.
//! Handles: plain classes, NestJS services/controllers, DI, routing.

mod struct_gen;
mod constructor;
mod method;
mod routing;
mod mutation;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use swc_ecma_ast::*;

use super::helpers::{to_snake_case, is_primitive_type};
use super::interface::RustGenerator;
use super::type_mapper;

/// Context passed between class sub-modules.
pub(crate) struct ClassContext {
    pub struct_name: proc_macro2::Ident,
    pub is_controller: bool,
    pub is_service: bool,
    pub controller_path: String,
    pub fields: Vec<(String, TokenStream, bool)>,  // (name, type, is_optional)
    pub dependency_fields: HashSet<String>,
    pub state_fields: HashSet<String>,
    pub generic_params: HashSet<String>,
    pub has_generics: bool,
}

impl RustGenerator {
    pub fn process_class_decl(&mut self, n: &ClassDecl) {
        let class_name = n.ident.sym.as_ref();
        let struct_name = format_ident!("{}", class_name);

        // Phase 1: Analyze class metadata
        let ctx = self.analyze_class(n, &struct_name);

        // Phase 2: Generate struct definition
        let struct_def = struct_gen::generate_struct(&ctx);

        // Phase 3: Generate constructor (new/new_di)
        let constructor = constructor::generate(&self, n, &ctx);

        // Phase 4: Generate methods
        let methods = method::generate_all(&self, n, &ctx);

        // Phase 5: Generate routing (if controller)
        let routing = if ctx.is_controller {
            routing::generate_router(&ctx)
        } else {
            quote! {}
        };

        // Phase 6: Generate FromRequestParts (if service/controller)
        let from_request = if ctx.is_service || ctx.is_controller {
            routing::generate_from_request_parts(&ctx)
        } else {
            quote! {}
        };

        // Combine all generated code
        // NOTE: RustGenerator uses `self.code: String`, NOT Vec<TokenStream>.
        // All TokenStreams must be converted via .to_string() before appending.
        self.code.push_str(&struct_def.to_string());
        self.code.push('\n');
        let impl_block = quote! {
            impl #struct_name {
                #constructor
                #(#methods)*
                #routing
            }
        };
        self.code.push_str(&impl_block.to_string());
        self.code.push('\n');
        self.code.push_str(&from_request.to_string());
    }

    fn analyze_class(&self, n: &ClassDecl, struct_name: &proc_macro2::Ident) -> ClassContext {
        // Extract metadata from class: decorators, fields, dependencies, generics
        // ~100 lines of analysis logic
        // Returns ClassContext
    }
}
```

- [ ] **Step 2: Create struct_gen.rs (~80 lines)**

Generates the `struct` definition with `#[derive(...)]` attributes.

- [ ] **Step 3: Create constructor.rs (~150 lines)**

Handles constructor parameter extraction, DI detection, field initialization.
Introduce `ConstructorContext` struct to replace the 8-parameter function:

```rust
pub(crate) struct ConstructorContext<'a> {
    pub class_context: &'a ClassContext,
    pub constructor: &'a Constructor,
    pub class_fields: &'a [(String, bool)],
}
```

- [ ] **Step 4: Create method.rs (~120 lines)**

Method transpilation with `&self` vs `&mut self` detection.

- [ ] **Step 5: Create routing.rs (~100 lines)**

Axum router generation, HTTP method handlers, FromRequestParts.

- [ ] **Step 6: Create mutation.rs (~60 lines)**

`body_mutates_self`, `stmt_mutates_self`, `expr_mutates_self`.

- [ ] **Step 7: Delete old class.rs, update mod.rs**

```bash
rm crates/tyrus_codegen/src/convert/class.rs
```

- [ ] **Step 8: Verify build and line counts**

```bash
cargo build --workspace && cargo clippy --workspace
wc -l crates/tyrus_codegen/src/convert/class/*.rs
```
Expected: Each file < 200 lines.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "refactor: decompose class.rs into class/ module — 1033 lines → 6 focused files"
```

---

### Task 10: Split Orchestrator

**Files:**
- Create: `crates/tyrus_orchestrator/src/pipeline.rs`
- Create: `crates/tyrus_orchestrator/src/scaffold.rs`
- Create: `crates/tyrus_orchestrator/src/format.rs`
- Modify: `crates/tyrus_orchestrator/src/lib.rs`

- [ ] **Step 1: Create pipeline.rs (~200 lines)**

Move `build_project` core logic.

- [ ] **Step 2: Create scaffold.rs (~150 lines)**

Move `generate_main_rs`, `generate_cargo_toml`, `generate_mod_rs`.
Replace string concatenation with `quote!` macros where possible.
Fix the `.expect()` calls by generating `main() -> Result<()>`.

- [ ] **Step 3: Create format.rs (~50 lines)**

Move `format_code` and `get_app_error_code`.

- [ ] **Step 4: Slim down lib.rs to ~100 lines**

Keep only `check()`, `build()`, and `build_project()` as thin wrappers.

- [ ] **Step 5: Verify build**

```bash
cargo build --workspace && cargo clippy --workspace
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor: split orchestrator into pipeline/scaffold/format — 505 lines → 4 focused files"
```

---

### Task 11: Final Clean Up (Post-Refactoring Verification)

> **NOTE:** Dead code removal (tyrus_ast, analyzer/graph.rs) was already done in Task 1 Step 3.
> This task handles only post-refactoring cleanup.

**Files:**
- Modify: `crates/tyrus_orchestrator/src/lib.rs` (remove `pipeline()` stub if still present)

- [ ] **Step 1: Remove orchestrator `pipeline()` stub**

If `pipeline()` stub still exists in orchestrator, remove it.

- [ ] **Step 2: Run full format + clippy + build**

```bash
cargo fmt --all
cargo clippy --workspace
cargo build --workspace
```
Expected: SUCCESS — all refactoring preserved behavior, no violations.

- [ ] **Step 3: Verify line counts meet targets**

```bash
find crates/ -name "*.rs" -exec wc -l {} + | sort -n
```
Expected: No file exceeds 400 lines.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: post-refactoring cleanup and verification"
```

---

## Chunk 4: Tier 1 Tests — Basic TypeScript

> From this point forward, each task follows the same TDD pattern:
> 1. Create fixture file with TypeScript input
> 2. Write snapshot test to capture transpilation output
> 3. Write unit test for specific codegen behavior
> 4. Optionally write compilation test (only for complex scenarios)
> 5. Commit

### Task 12: Variable Declarations & Primitive Types

**Files:**
- Create: `tests/fixtures/tier1/variables.ts`
- Modify: `tests/src/snapshot/tier1.rs`
- Modify: `tests/src/unit/expr.rs`

- [ ] **Step 1: Create fixture**

Create `tests/fixtures/tier1/variables.ts`:
```typescript
const name: string = "Alice";
const age: number = 30;
const active: boolean = true;
const pi: number = 3.14159;
let count: number = 0;
const greeting: string = "Hello, World!";
```

- [ ] **Step 2: Write snapshot test**

In `tests/src/snapshot/tier1.rs`:
```rust
use crate::helpers::transpile_fixture;

#[test]
fn test_snapshot_variables() {
    let rust = transpile_fixture("tier1/variables");
    insta::assert_snapshot!(rust);
}
```

- [ ] **Step 3: Run test to generate initial snapshot**

```bash
cargo test -p integration_tests snapshot::tier1::test_snapshot_variables
cargo insta review
```
Expected: New snapshot created, review and accept.

- [ ] **Step 4: Write unit tests**

In `tests/src/unit/expr.rs`:
```rust
use crate::helpers::transpile;

#[test]
fn test_const_string_declaration() {
    let rust = transpile("const name: string = \"Alice\";");
    assert!(rust.contains("let name: String = \"Alice\".to_string()"));
}

#[test]
fn test_const_number_declaration() {
    let rust = transpile("const x: number = 42;");
    assert!(rust.contains("let x: f64 = 42"));
}

#[test]
fn test_const_boolean_declaration() {
    let rust = transpile("const flag: boolean = true;");
    assert!(rust.contains("let flag: bool = true"));
}

#[test]
fn test_let_declaration() {
    let rust = transpile("let count: number = 0;");
    assert!(rust.contains("let mut count: f64 = 0"));
}
```

- [ ] **Step 5: Run all tests**

```bash
cargo test -p integration_tests -- tier1
```
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add tests/fixtures/tier1/variables.ts tests/src/snapshot/tier1.rs tests/src/unit/expr.rs
git commit -m "test: tier1 — variable declarations and primitive types"
```

---

### Task 13: Math Operations

**Files:**
- Create: `tests/fixtures/tier1/math_ops.ts`
- Modify: `tests/src/snapshot/tier1.rs`
- Modify: `tests/src/unit/expr.rs`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier1/math_ops.ts
function add(a: number, b: number): number {
    return a + b;
}

function subtract(a: number, b: number): number {
    return a - b;
}

function multiply(a: number, b: number): number {
    return a * b;
}

function divide(a: number, b: number): number {
    return a / b;
}

function modulo(a: number, b: number): number {
    return a % b;
}

function complex(x: number, y: number, z: number): number {
    return (x + y) * z - x / y;
}
```

- [ ] **Step 2: Write snapshot test**

```rust
#[test]
fn test_snapshot_math_ops() {
    let rust = transpile_fixture("tier1/math_ops");
    insta::assert_snapshot!(rust);
}
```

- [ ] **Step 3: Write unit tests for binary operators**

```rust
#[test]
fn test_addition() {
    let rust = transpile("function add(a: number, b: number): number { return a + b; }");
    assert!(rust.contains("a + b"));
}

#[test]
fn test_subtraction() {
    let rust = transpile("function sub(a: number, b: number): number { return a - b; }");
    assert!(rust.contains("a - b"));
}

#[test]
fn test_string_concat_uses_format() {
    let rust = transpile("function greet(name: string): string { return \"Hello \" + name; }");
    assert!(rust.contains("format!"));
}
```

- [ ] **Step 4: Run tests, review snapshots, commit**

```bash
cargo test -p integration_tests -- tier1
cargo insta review
git add -A && git commit -m "test: tier1 — math operations and binary expressions"
```

---

### Task 14: String Operations

**Files:**
- Create: `tests/fixtures/tier1/string_ops.ts`
- Modify: `tests/src/snapshot/tier1.rs`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier1/string_ops.ts
function greet(name: string): string {
    return `Hello, ${name}!`;
}

function fullName(first: string, last: string): string {
    return `${first} ${last}`;
}

const message: string = "Hello, World!";
```

- [ ] **Step 2: Write snapshot + unit tests**
- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier1 — string operations and template literals"
```

---

### Task 15: Functions

**Files:**
- Create: `tests/fixtures/tier1/functions.ts`
- Modify: `tests/src/snapshot/tier1.rs`
- Modify: `tests/src/unit/stmt.rs`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier1/functions.ts
function square(x: number): number {
    return x * x;
}

function isPositive(n: number): boolean {
    return n > 0;
}

function formatUser(name: string, age: number): string {
    return `${name} is ${age} years old`;
}
```

- [ ] **Step 2: Write tests for function signatures**

```rust
#[test]
fn test_function_with_params() {
    let rust = transpile("function add(a: number, b: number): number { return a + b; }");
    assert!(rust.contains("fn add(a: f64, b: f64) -> f64"));
}

#[test]
fn test_function_returns_string() {
    let rust = transpile("function greet(name: string): string { return name; }");
    assert!(rust.contains("fn greet(name: String) -> String"));
}
```

- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier1 — function declarations with params and return types"
```

---

### Task 16: Control Flow

**Files:**
- Create: `tests/fixtures/tier1/control_flow.ts`
- Modify: `tests/src/snapshot/tier1.rs`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier1/control_flow.ts
function abs(x: number): number {
    if (x < 0) {
        return -x;
    } else {
        return x;
    }
}

function countdown(n: number): number {
    let result: number = 0;
    let i: number = n;
    while (i > 0) {
        result = result + i;
        i = i - 1;
    }
    return result;
}

function classify(x: number): string {
    if (x > 0) {
        return "positive";
    } else if (x < 0) {
        return "negative";
    } else {
        return "zero";
    }
}
```

- [ ] **Step 2: Write tests**
- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier1 — control flow (if/else, while)"
```

---

### Task 17: console.log

**Files:**
- Create: `tests/fixtures/tier1/console.ts`
- Modify: `tests/src/unit/stdlib.rs`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier1/console.ts
const x: number = 42;
console.log(x);
console.error("something went wrong");
```

- [ ] **Step 2: Write unit test**

```rust
// tests/src/unit/stdlib.rs
use crate::helpers::transpile;

#[test]
fn test_console_log() {
    let rust = transpile("console.log(42);");
    assert!(rust.contains("println!"));
}

#[test]
fn test_console_error() {
    let rust = transpile("console.error(\"error\");");
    assert!(rust.contains("eprintln!"));
}
```

- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier1 — console.log and console.error mapping"
```

---

### Task 18: Tier 1 Compilation Verification

**Files:**
- Modify: `tests/src/compilation/tier1.rs`

- [ ] **Step 1: Write batch compilation test**

```rust
// tests/src/compilation/tier1.rs
use tyrus_test_utils::assert_rust_compiles;
use crate::helpers::transpile_fixture;

/// Batch compile ALL tier1 fixtures in a single test.
/// This is intentionally a single test to minimize cargo check invocations.
#[test]
fn test_tier1_compiles() {
    let fixtures = [
        "tier1/variables",
        "tier1/math_ops",
        "tier1/string_ops",
        "tier1/functions",
        "tier1/control_flow",
        "tier1/console",
    ];

    for fixture in &fixtures {
        let rust = transpile_fixture(fixture);
        assert_rust_compiles(&rust);
    }
}
```

- [ ] **Step 2: Run compilation test**

```bash
cargo test -p integration_tests compilation::tier1::test_tier1_compiles
```
Expected: PASS (all tier1 code compiles)

- [ ] **Step 3: Commit**

```bash
git commit -m "test: tier1 — batch compilation verification for all basic TypeScript features"
```

---

## Chunk 5: Tier 2 Tests — Intermediate TypeScript

### Task 19: Interfaces → Structs

**Files:**
- Create: `tests/fixtures/tier2/interfaces.ts`
- Create: `tests/src/snapshot/tier2.rs`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier2/interfaces.ts
interface User {
    name: string;
    age: number;
    email: string;
    active: boolean;
}

interface Product {
    id: number;
    title: string;
    price: number;
    description?: string;
}

interface ApiResponse {
    data: User[];
    total: number;
    success: boolean;
}
```

- [ ] **Step 2: Write snapshot test**

```rust
// tests/src/snapshot/tier2.rs
use crate::helpers::transpile_fixture;

#[test]
fn test_snapshot_interfaces() {
    let rust = transpile_fixture("tier2/interfaces");
    insta::assert_snapshot!(rust);
}
```

- [ ] **Step 3: Write unit tests**

```rust
#[test]
fn test_interface_generates_struct() {
    let rust = transpile("interface User { name: string; age: number; }");
    assert!(rust.contains("struct User"));
    assert!(rust.contains("name: String"));
    assert!(rust.contains("age: f64"));
}

#[test]
fn test_interface_has_serde_derive() {
    let rust = transpile("interface User { name: string; }");
    assert!(rust.contains("Serialize"));
    assert!(rust.contains("Deserialize"));
}

#[test]
fn test_optional_field() {
    let rust = transpile("interface Config { debug?: boolean; }");
    assert!(rust.contains("Option<bool>"));
}
```

- [ ] **Step 4: Run, review, commit**

```bash
git commit -m "test: tier2 — interfaces with struct generation and serde derives"
```

---

### Task 20: Type Aliases & String Union Enums

**Files:**
- Create: `tests/fixtures/tier2/type_aliases.ts`
- Modify: `tests/src/snapshot/tier2.rs`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier2/type_aliases.ts
type ID = string;
type NumberList = number[];

type Status = "active" | "inactive" | "pending";
type Priority = "low" | "medium" | "high" | "critical";
```

- [ ] **Step 2: Write tests verifying enum generation**

```rust
#[test]
fn test_string_union_generates_enum() {
    let rust = transpile("type Status = \"active\" | \"inactive\" | \"pending\";");
    assert!(rust.contains("enum Status"));
    assert!(rust.contains("Active"));
    assert!(rust.contains("Inactive"));
    assert!(rust.contains("Pending"));
}
```

- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier2 — type aliases and string union to enum conversion"
```

---

### Task 21: Arrays

**Files:**
- Create: `tests/fixtures/tier2/arrays.ts`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier2/arrays.ts
const numbers: number[] = [1, 2, 3, 4, 5];

function doubleAll(nums: number[]): number[] {
    return nums.map((n: number) => n * 2);
}

function evens(nums: number[]): number[] {
    return nums.filter((n: number) => n % 2 === 0);
}

function sum(nums: number[]): number {
    let total: number = 0;
    nums.forEach((n: number) => {
        total = total + n;
    });
    return total;
}
```

- [ ] **Step 2: Write tests for array method mapping**
- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier2 — arrays with map, filter, forEach"
```

---

### Task 22: Classes

**Files:**
- Create: `tests/fixtures/tier2/classes.ts`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier2/classes.ts
class Calculator {
    private result: number;

    constructor() {
        this.result = 0;
    }

    add(value: number): number {
        this.result = this.result + value;
        return this.result;
    }

    getResult(): number {
        return this.result;
    }

    reset(): void {
        this.result = 0;
    }
}
```

- [ ] **Step 2: Write tests for class → struct + impl**

```rust
#[test]
fn test_class_generates_struct() {
    let rust = transpile("class Dog { name: string; constructor(name: string) { this.name = name; } }");
    assert!(rust.contains("struct Dog"));
    assert!(rust.contains("impl Dog"));
    assert!(rust.contains("fn new"));
}
```

- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier2 — classes with constructors and methods"
```

---

### Task 23: Async/Await

**Files:**
- Create: `tests/fixtures/tier2/async_await.ts`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier2/async_await.ts
interface UserData {
    id: number;
    name: string;
}

async function fetchUser(id: number): Promise<UserData> {
    const response = await fetch(`https://api.example.com/users/${id}`);
    const data: UserData = await response.json();
    return data;
}

async function processUser(id: number): Promise<string> {
    const user: UserData = await fetchUser(id);
    return user.name;
}
```

- [ ] **Step 2: Write tests**

```rust
#[test]
fn test_async_function() {
    let rust = transpile("async function getData(): Promise<string> { return \"hello\"; }");
    assert!(rust.contains("async fn"));
    assert!(rust.contains("Result<"));
}
```

- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier2 — async/await with Promise → Result mapping"
```

---

### Task 24: Tier 2 Compilation Verification

- [ ] **Step 1: Write batch compilation test**

```rust
// tests/src/compilation/tier2.rs
#[test]
fn test_tier2_compiles() {
    let fixtures = [
        "tier2/interfaces",
        "tier2/type_aliases",
        "tier2/arrays",
        "tier2/classes",
        "tier2/async_await",
    ];
    for fixture in &fixtures {
        let rust = transpile_fixture(fixture);
        assert_rust_compiles(&rust);
    }
}
```

- [ ] **Step 2: Run, commit**

```bash
git commit -m "test: tier2 — batch compilation verification"
```

---

## Chunk 6: Tier 3 Tests — Advanced TypeScript

### Task 25: Generics

**Files:**
- Create: `tests/fixtures/tier3/generics.ts`
- Create: `tests/src/snapshot/tier3.rs`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier3/generics.ts
interface Container<T> {
    value: T;
    label: string;
}

class Wrapper<T> {
    private data: T;

    constructor(data: T) {
        this.data = data;
    }

    get(): T {
        return this.data;
    }
}

function identity<T>(value: T): T {
    return value;
}
```

- [ ] **Step 2: Write tests**
- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier3 — generics (interface, class, function)"
```

---

### Task 26: Optional Chaining & Nullish Coalescing

**Files:**
- Create: `tests/fixtures/tier3/optional_chaining.ts`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier3/optional_chaining.ts
interface Profile {
    name: string;
    address?: string;
}

interface User {
    id: number;
    profile?: Profile;
}

function getUserName(user: User): string {
    const name: string = user.profile?.name ?? "Unknown";
    return name;
}

function getAddress(user: User): string {
    return user.profile?.address ?? "No address";
}
```

- [ ] **Step 2: Write tests**
- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier3 — optional chaining and nullish coalescing"
```

---

### Task 27: Destructuring

**Files:**
- Create: `tests/fixtures/tier3/destructuring.ts`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier3/destructuring.ts
interface Point {
    x: number;
    y: number;
}

function getCoords(point: Point): number {
    const { x, y } = point;
    return x + y;
}

function getFirst(items: number[]): number {
    const [first] = items;
    return first;
}
```

- [ ] **Step 2: Write tests**
- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier3 — object and array destructuring"
```

---

### Task 28: Advanced Array/String Methods

**Files:**
- Create: `tests/fixtures/tier3/advanced_methods.ts`
- Create: `tests/fixtures/tier3/string_methods.ts`

- [ ] **Step 1: Create fixtures**

```typescript
// tests/fixtures/tier3/advanced_methods.ts
function findFirst(nums: number[]): number {
    const found: number = nums.find((n: number) => n > 10) ?? 0;
    return found;
}

function hasLarge(nums: number[]): boolean {
    return nums.some((n: number) => n > 100);
}

function allPositive(nums: number[]): boolean {
    return nums.every((n: number) => n > 0);
}
```

```typescript
// tests/fixtures/tier3/string_methods.ts
function process(input: string): string {
    const upper: string = input.toUpperCase();
    const trimmed: string = upper.trim();
    return trimmed;
}

function contains(haystack: string, needle: string): boolean {
    return haystack.includes(needle);
}

function splitAndJoin(input: string): string {
    const parts: string[] = input.split(",");
    return parts.join(" - ");
}
```

- [ ] **Step 2: Write tests**
- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier3 — advanced array methods and string methods"
```

---

### Task 29: Math & JSON Stdlib

**Files:**
- Create: `tests/fixtures/tier3/math_stdlib.ts`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier3/math_stdlib.ts
function clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
}

function roundToTwo(n: number): number {
    return Math.round(n * 100) / 100;
}

function distance(x: number, y: number): number {
    return Math.abs(x - y);
}
```

- [ ] **Step 2: Write tests**
- [ ] **Step 3: Run, review, commit**

```bash
git commit -m "test: tier3 — Math stdlib and JSON methods"
```

---

### Task 30: Ternary Expressions

**Files:**
- Create: `tests/fixtures/tier3/ternary.ts`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier3/ternary.ts
function maxOf(a: number, b: number): number {
    return a > b ? a : b;
}

function label(active: boolean): string {
    return active ? "Active" : "Inactive";
}
```

- [ ] **Step 2: Write tests**
- [ ] **Step 3: Commit**

```bash
git commit -m "test: tier3 — ternary expressions"
```

---

### Task 31: Tier 3 Compilation Verification

- [ ] **Step 1: Write batch compilation test**
- [ ] **Step 2: Run, commit**

```bash
git commit -m "test: tier3 — batch compilation verification"
```

---

## Chunk 7: Tier 4 Tests — NestJS

### Task 32: @Injectable Services

**Files:**
- Create: `tests/fixtures/tier4/injectable_service.ts`
- Create: `tests/src/snapshot/tier4_nestjs.rs`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier4/injectable_service.ts
import { Injectable } from "@nestjs/common";

interface User {
    id: number;
    name: string;
    email: string;
}

@Injectable()
class UsersService {
    private users: User[];

    constructor() {
        this.users = [];
    }

    findAll(): User[] {
        return this.users;
    }

    findById(id: number): User {
        return this.users.find((user: User) => user.id === id);
    }

    create(name: string, email: string): User {
        const user: User = {
            id: this.users.length + 1,
            name: name,
            email: email,
        };
        this.users.push(user);
        return user;
    }
}
```

- [ ] **Step 2: Write snapshot test**

```rust
#[test]
fn test_snapshot_injectable_service() {
    let rust = transpile_fixture("tier4/injectable_service");
    insta::assert_snapshot!(rust);
}
```

- [ ] **Step 3: Write unit test for Arc<Mutex<>> wrapping**

```rust
#[test]
fn test_injectable_uses_arc_mutex() {
    let rust = transpile_fixture("tier4/injectable_service");
    assert!(rust.contains("Arc<Mutex<"));
    assert!(!rust.contains("&mut self"));
}
```

- [ ] **Step 4: Commit**

```bash
git commit -m "test: tier4 — @Injectable service with DI and Arc<Mutex<T>>"
```

---

### Task 33: @Controller with Routing

**Files:**
- Create: `tests/fixtures/tier4/controller.ts`

- [ ] **Step 1: Create fixture**

```typescript
// tests/fixtures/tier4/controller.ts
import { Controller, Get, Post, Body, Param } from "@nestjs/common";
import { Injectable } from "@nestjs/common";

interface CreateUserDto {
    name: string;
    email: string;
}

interface User {
    id: number;
    name: string;
    email: string;
}

@Injectable()
class UsersService {
    private users: User[];

    constructor() {
        this.users = [];
    }

    findAll(): User[] {
        return this.users;
    }

    create(dto: CreateUserDto): User {
        const user: User = {
            id: this.users.length + 1,
            name: dto.name,
            email: dto.email,
        };
        this.users.push(user);
        return user;
    }
}

@Controller("/users")
class UsersController {
    constructor(private usersService: UsersService) {}

    @Get("/")
    findAll(): User[] {
        return this.usersService.findAll();
    }

    @Post("/")
    create(@Body() dto: CreateUserDto): User {
        return this.usersService.create(dto);
    }
}
```

- [ ] **Step 2: Write tests for Axum routing generation**

```rust
#[test]
fn test_controller_generates_router() {
    let rust = transpile_fixture("tier4/controller");
    assert!(rust.contains("fn router()"));
    assert!(rust.contains("axum::routing::get"));
    assert!(rust.contains("axum::routing::post"));
}

#[test]
fn test_controller_handler_uses_json_extractor() {
    let rust = transpile_fixture("tier4/controller");
    assert!(rust.contains("Json("));
}
```

- [ ] **Step 3: Commit**

```bash
git commit -m "test: tier4 — @Controller with @Get/@Post routing and Axum mapping"
```

---

### Task 34: Full NestJS Project Compilation

**Files:**
- Create: `tests/fixtures/tier4/full_project/` (multi-file)
- Create: `tests/src/compilation/nestjs.rs`

- [ ] **Step 1: Create multi-file NestJS fixture**

```
tests/fixtures/tier4/full_project/
  src/
    app.module.ts
    users/
      users.module.ts
      users.service.ts
      users.controller.ts
      dto/
        create-user.dto.ts
```

Each file follows NestJS patterns with proper imports and decorators.

- [ ] **Step 2: Write compilation test**

```rust
// tests/src/compilation/nestjs.rs
use std::path::Path;

#[test]
fn test_nestjs_full_project_compiles() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/tier4/full_project/src");
    let output = tempfile::tempdir().expect("tempdir");

    tyrus_orchestrator::build_project(input, output.path().to_path_buf())
        .expect("build_project failed");

    // Verify generated Cargo.toml exists
    assert!(output.path().join("Cargo.toml").exists());

    // Verify generated code compiles
    let status = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(output.path())
        .env("CARGO_TARGET_DIR", tyrus_test_utils::shared_target_dir())
        .status()
        .expect("cargo check");

    assert!(status.success(), "Generated NestJS project failed to compile");
}
```

- [ ] **Step 3: Run test, fix any codegen issues**

```bash
cargo test -p integration_tests compilation::nestjs
```

- [ ] **Step 4: Commit**

```bash
git commit -m "test: tier4 — full NestJS project multi-file compilation"
```

---

## Chunk 8: Analyzer & CLI Enhancements

### Task 35: Strengthen tyrus_analyzer

**Files:**
- Modify: `crates/tyrus_analyzer/src/lints.rs`

Current lint rules reject: `var`, `any`, `eval`, `do-while`, `for`, `for-of`, `for-in`, `try-catch`, `switch`.

**Add new lint rules:**

- [ ] **Step 1: Write failing tests for new lint rules**

```rust
// tests/src/unit/analyzer.rs (new file)
use tyrus_parser;
use tyrus_analyzer::Analyzer;

#[test]
fn test_rejects_var_declaration() {
    let result = analyze_code("var x = 10;");
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].to_string().contains("var"));
}

#[test]
fn test_accepts_const_declaration() {
    let result = analyze_code("const x: number = 10;");
    assert!(result.errors.is_empty());
}

#[test]
fn test_rejects_any_type() {
    let result = analyze_code("const x: any = null;");
    assert!(!result.errors.is_empty());
}

#[test]
fn test_rejects_eval() {
    let result = analyze_code("eval('code');");
    assert!(!result.errors.is_empty());
}

fn analyze_code(ts: &str) -> tyrus_analyzer::AnalysisResult {
    let tmp = tempfile::NamedTempFile::with_suffix(".ts").unwrap();
    std::fs::write(tmp.path(), ts).unwrap();
    let program = tyrus_parser::parse(tmp.path()).unwrap();
    Analyzer::analyze(&program, ts.to_string(), "test.ts".to_string())
}
```

- [ ] **Step 2: Run tests to verify existing rules work**

```bash
cargo test -p integration_tests unit::analyzer
```

- [ ] **Step 3: Add performance benchmarks (optional)**

Consider adding `criterion` benchmarks for analyzer speed on large files.

- [ ] **Step 4: Commit**

```bash
git commit -m "test: analyzer — unit tests for lint rules (var, any, eval rejection)"
```

---

### Task 36: Enhance CLI

**Files:**
- Modify: `crates/tyrus_cli/src/main.rs`

- [ ] **Step 1: Add new CLI commands**

```rust
// Potential enhancements:
// - `tyrus lint <file>` — run only lint analysis
// - `tyrus fmt <file>` — format generated output
// - `tyrus watch <dir>` — watch mode for development
// - Better error output with miette rich diagnostics
// - Progress indicators for multi-file projects
```

- [ ] **Step 2: Write CLI integration tests**

```rust
#[test]
fn test_cli_check_valid_file() {
    let cmd = assert_cmd::Command::cargo_bin("tyrus").unwrap();
    cmd.arg("check")
        .arg("tests/fixtures/tier1/variables.ts")
        .assert()
        .success();
}

#[test]
fn test_cli_build_valid_file() {
    let cmd = assert_cmd::Command::cargo_bin("tyrus").unwrap();
    let tmp = tempfile::tempdir().unwrap();
    cmd.arg("build")
        .arg("tests/fixtures/tier1/variables.ts")
        .arg("--output")
        .arg(tmp.path())
        .assert()
        .success();
}
```

- [ ] **Step 3: Commit**

```bash
git commit -m "test: CLI — integration tests for check and build commands"
```

---

## Execution Summary

### Phase Ordering

| Phase | Chunk | Tasks | Focus | Duration Est. |
|-------|-------|-------|-------|---------------|
| 0 | 1 | 1-3 | Clean slate + strict rules + fix violations | First |
| 1 | 2-3 | 4-11 | Codegen refactoring (func.rs, class.rs, orchestrator) | Second |
| 2 | 4 | 12-18 | Tier 1: Basic TypeScript | Third |
| 3 | 5 | 19-24 | Tier 2: Intermediate TypeScript | Fourth |
| 4 | 6 | 25-31 | Tier 3: Advanced TypeScript | Fifth |
| 5 | 7 | 32-34 | Tier 4: NestJS | Sixth |
| 6 | 8 | 35-36 | Analyzer + CLI | Seventh |

### Quality Gates

After each phase:
1. `cargo fmt -- --check` — Formatting passes
2. `cargo clippy --workspace` — No warnings (strict mode)
3. `cargo test --workspace` — All tests pass
4. All files < 400 lines
5. All functions < 50 lines
6. Max nesting depth: 4 levels
7. Zero `todo!()`, `unwrap()`, `expect()`, `panic!()` in library code

### Test Performance Targets

| Test Category | Count | Target Time |
|---------------|-------|-------------|
| Unit tests | ~40 | < 5s |
| Snapshot tests | ~20 | < 10s |
| Compilation tests (batched) | 4 batches | < 60s |
| **Total** | ~64 | **< 75s** |

### Key Architectural Decisions

1. **Unit tests first**: Test `convert_expr`, `convert_stmt` directly — no compilation overhead
2. **Snapshot tests second**: Full transpilation output compared via `insta`
3. **Compilation tests batched**: Group fixtures by tier, one `cargo check` per tier
4. **Shared CARGO_TARGET_DIR**: All compilation tests share dependency cache
5. **cargo-nextest**: Parallel test execution for maximum speed
6. **No `todo!()` allowed**: Use `compile_error!()` in generated code instead — surfaces at compile time, not runtime
