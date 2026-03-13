# Milestone 13A: Semantic Equivalence Infrastructure + Basic Coverage

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a testing infrastructure that runs BOTH the original TypeScript (Node.js) and the generated Rust, comparing stdout to prove semantic equivalence. Then use it to verify and fix the most basic transpilation patterns.

**Architecture:** New test helper `assert_output_equivalent(ts_code)` that: (1) runs TS via `node --experimental-strip-types`, (2) transpiles TS→Rust, (3) wraps Rust in `fn main()`, (4) compiles + runs Rust binary, (5) asserts stdout is identical. Bug fixes for `const`/`let` immutability and unary negation are prerequisites since they affect output correctness.

**Tech Stack:** Rust (`std::process::Command`), Node.js (runtime), `tempfile`, `cargo build`/`cargo run`, `tyrus_orchestrator::build()`.

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `crates/tyrus_test_utils/src/lib.rs` | Modify | Add `assert_output_equivalent()` and `compile_and_run_rust()` |
| `tests/src/equivalence/mod.rs` | Create | Module declarations for equivalence tests |
| `tests/src/equivalence/basic.rs` | Create | Basic arithmetic, variables, function calls |
| `tests/src/equivalence/strings.rs` | Create | String methods: includes, replace, split, toUpperCase, toLowerCase, trim |
| `tests/src/equivalence/arrays.rs` | Create | Array methods: map, filter, forEach, push, find, join |
| `tests/src/equivalence/console.rs` | Create | console.log formatting equivalence |
| `tests/src/lib.rs` | Modify | Add `mod equivalence;` |
| `crates/tyrus_codegen/src/convert/stmt.rs` | Modify | Fix const→let (immutable) vs let→let mut |
| `crates/tyrus_codegen/src/convert/expr/mod.rs` | Modify | Add Expr::Unary dispatch |
| `crates/tyrus_codegen/src/convert/expr/misc.rs` | Modify | Add convert_unary_expr implementation |
| `crates/tyrus_codegen/src/stdlib/string.rs` | Modify | Fix trim() → .trim().to_string(), add startsWith, endsWith, substring |
| `crates/tyrus_codegen/src/stdlib/array.rs` | Modify | Add includes, indexOf, length |

---

## Chunk 1: Semantic Equivalence Testing Infrastructure

### Task 1: Add `compile_and_run_rust()` to test utils

**Files:**
- Modify: `crates/tyrus_test_utils/src/lib.rs`

This function takes Rust code, wraps it in a binary project with `fn main()`, compiles, runs, and returns stdout.

- [ ] **Step 1: Write the `compile_and_run_rust` function**

Add to `crates/tyrus_test_utils/src/lib.rs`:

```rust
/// Compiles Rust code as a binary and runs it, returning stdout.
/// The code must contain a `fn main()` or be wrapped by the caller.
///
/// # Panics
/// Panics if compilation or execution fails.
pub fn compile_and_run_rust(code: &str) -> String {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();
    let shared_target = get_shared_target_dir();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");

    // Binary project (main.rs, not lib.rs)
    let cargo_toml = r#"
[package]
name = "tyrus_app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "tyrus_app"
path = "src/main.rs"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
rand = "0.8"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml)
        .expect("Failed to write Cargo.toml");

    let wrapped_code = format!(
        "#![allow(dead_code, unused_variables, unused_imports, unused_mut)]\n{}",
        code
    );
    fs::write(src_dir.join("main.rs"), &wrapped_code)
        .expect("Failed to write main.rs");

    // Build
    let build_output = Command::new("cargo")
        .args(["build", "--quiet"])
        .env("CARGO_TARGET_DIR", shared_target)
        .current_dir(project_path)
        .output()
        .expect("Failed to execute cargo build");

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        panic!(
            "\n=== RUST BUILD FAILED ===\nCODE:\n{}\n\nSTDERR:\n{}",
            code, stderr
        );
    }

    // Run
    let bin_path = shared_target.join("debug").join("tyrus_app");
    let run_output = Command::new(&bin_path)
        .output()
        .expect("Failed to run compiled binary");

    if !run_output.status.success() {
        let stderr = String::from_utf8_lossy(&run_output.stderr);
        panic!(
            "\n=== RUST EXECUTION FAILED ===\nCODE:\n{}\n\nSTDERR:\n{}",
            code, stderr
        );
    }

    String::from_utf8_lossy(&run_output.stdout).to_string()
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p tyrus_test_utils`
Expected: compiles without errors

---

### Task 2: Add `run_node()` helper to test utils

**Files:**
- Modify: `crates/tyrus_test_utils/src/lib.rs`

- [ ] **Step 1: Write the `run_node` function**

```rust
/// Runs TypeScript code using Node.js and returns stdout.
/// Uses --experimental-strip-types for native TS support (Node 22+).
/// Falls back to treating code as JS if flag is unavailable.
///
/// # Panics
/// Panics if Node.js is not installed or execution fails.
pub fn run_node(ts_code: &str) -> String {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ts_file = temp_dir.path().join("test.ts");
    fs::write(&ts_file, ts_code).expect("Failed to write TS file");

    // Try with --experimental-strip-types first (Node 22+)
    let output = Command::new("node")
        .args(["--experimental-strip-types", ts_file.to_str().unwrap_or_default()])
        .output()
        .expect("Node.js not found. Install Node.js to run equivalence tests.");

    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout).to_string();
    }

    // Fallback: try as plain JS (write as .js without type annotations)
    let js_file = temp_dir.path().join("test.js");
    fs::write(&js_file, ts_code).expect("Failed to write JS file");
    let output = Command::new("node")
        .arg(js_file.to_str().unwrap_or_default())
        .output()
        .expect("Failed to run Node.js");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "\n=== NODE EXECUTION FAILED ===\nCODE:\n{}\n\nSTDERR:\n{}",
            ts_code, stderr
        );
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p tyrus_test_utils`

---

### Task 3: Add `assert_output_equivalent()` integration helper

**Files:**
- Modify: `tests/src/helpers.rs`

This is the main test helper that combines transpilation + Node execution + Rust execution + comparison.

- [ ] **Step 1: Write the helper**

Add to `tests/src/helpers.rs`:

```rust
/// Asserts that the transpiled Rust code produces identical stdout to the
/// original TypeScript when run with Node.js.
///
/// The TypeScript code MUST:
/// - Contain function declarations (not top-level statements)
/// - Have a `main()` function that calls the other functions
/// - Use `console.log()` for observable output
///
/// Example:
/// ```typescript
/// function add(a: number, b: number): number { return a + b; }
/// function main(): void { console.log(add(2, 3)); }
/// main();
/// ```
pub fn assert_output_equivalent(ts_code: &str) {
    // 1. Run TypeScript with Node.js
    let ts_output = tyrus_test_utils::run_node(ts_code);

    // 2. Transpile TS → Rust
    let rust_code = transpile(ts_code);

    // 3. Wrap in main() if needed (the transpiler generates functions but no main)
    // The TS code has main() which the transpiler will convert to fn main()
    // We need to make it the actual entry point
    let rust_binary = if rust_code.contains("fn main()") {
        rust_code.clone()
    } else {
        format!("{}\nfn main() {{}}", rust_code)
    };

    // 4. Compile + run Rust
    let rust_output = tyrus_test_utils::compile_and_run_rust(&rust_binary);

    // 5. Compare outputs
    assert_eq!(
        ts_output.trim(),
        rust_output.trim(),
        "\n╔══════════════════════════════════════════╗\n\
         ║  SEMANTIC EQUIVALENCE FAILURE             ║\n\
         ╚══════════════════════════════════════════╝\n\n\
         TypeScript output: {:?}\n\
         Rust output:       {:?}\n\n\
         Generated Rust code:\n{}\n",
        ts_output.trim(),
        rust_output.trim(),
        rust_code
    );
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p integration_tests`

- [ ] **Step 3: Commit**

```bash
git add crates/tyrus_test_utils/src/lib.rs tests/src/helpers.rs
git commit -m "feat(test): add semantic equivalence testing infrastructure"
```

---

### Task 4: Create equivalence test module structure

**Files:**
- Create: `tests/src/equivalence/mod.rs`
- Modify: `tests/src/lib.rs`

- [ ] **Step 1: Create module file**

`tests/src/equivalence/mod.rs`:
```rust
mod basic;
```

- [ ] **Step 2: Register in lib.rs**

Add `mod equivalence;` to `tests/src/lib.rs`.

- [ ] **Step 3: Write first smoke test**

`tests/src/equivalence/basic.rs`:
```rust
use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_simple_addition() {
    assert_output_equivalent(r#"
function add(a: number, b: number): number {
    return a + b;
}
function main(): void {
    console.log(add(2, 3));
}
main();
"#);
}
```

- [ ] **Step 4: Run test**

Run: `cargo test -p integration_tests equivalence::basic::test_equivalence_simple_addition -- --exact`

This test may FAIL if there are output format differences (e.g., Node prints `5`, Rust prints `5`... actually they might match since `println!("{}", 5f64)` prints `5`). Debug any differences.

- [ ] **Step 5: Commit**

```bash
git add tests/src/equivalence/ tests/src/lib.rs
git commit -m "test(equivalence): add first semantic equivalence smoke test"
```

---

## Chunk 2: Fix Critical Bugs Affecting Output

### Task 5: Fix `const` vs `let` immutability

**Context:** Both `const x = 1` and `let x = 1` currently produce `let mut x`. Fix: `const` → `let` (immutable), `let` → `let mut` (mutable). This affects generated code quality but not output — fixing it first ensures clean generated code.

**Files:**
- Test: `tests/src/equivalence/basic.rs`
- Modify: The file where `VarDecl` → `let mut` is emitted (need to find it)

- [ ] **Step 1: Find where VarDecl kind is processed**

Search for `let mut` or `VarDeclKind` in the codegen crate. Check `stmt.rs`, `interface.rs`, and `fn_decl.rs`.

Run: `grep -rn "let mut" crates/tyrus_codegen/src/convert/` to find all emission points.
Run: `grep -rn "VarDeclKind" crates/tyrus_codegen/src/convert/` to find kind checks.

- [ ] **Step 2: Write equivalence test**

```rust
#[test]
fn test_equivalence_const_vs_let() {
    assert_output_equivalent(r#"
function main(): void {
    const x: number = 10;
    let y: number = 20;
    console.log(x);
    console.log(y);
}
main();
"#);
}
```

- [ ] **Step 3: Fix the VarDecl emission**

Change the logic so that:
- `VarDeclKind::Const` → emit `let`
- `VarDeclKind::Let` → emit `let mut`
- `VarDeclKind::Var` → rejected by analyzer (never reaches codegen)

- [ ] **Step 4: Run tests — verify old tests still pass + new test passes**

Run: `cargo nextest run --workspace`

- [ ] **Step 5: Update any broken snapshots**

Run: `cargo insta review` (snapshots will change since `const` now produces `let` instead of `let mut`)
Run: `cargo insta accept`

- [ ] **Step 6: Commit**

```bash
git commit -m "fix(codegen): distinguish const (let) from let (let mut) declarations"
```

---

### Task 6: Fix unary expression support (`!`, `-`, `+`)

**Context:** `-x`, `!flag`, `+str` all produce `compile_error!("Tyrus: unsupported expression")`. This blocks basic arithmetic like `return -x` or `if (!done)`.

**Files:**
- Test: `tests/src/equivalence/basic.rs`
- Modify: `crates/tyrus_codegen/src/convert/expr/mod.rs`
- Modify: `crates/tyrus_codegen/src/convert/expr/misc.rs`

- [ ] **Step 1: Write equivalence test**

```rust
#[test]
fn test_equivalence_unary_negation() {
    assert_output_equivalent(r#"
function negate(x: number): number {
    return -x;
}
function main(): void {
    console.log(negate(5));
    console.log(negate(-3));
}
main();
"#);
}

#[test]
fn test_equivalence_logical_not() {
    assert_output_equivalent(r#"
function invert(x: boolean): boolean {
    return !x;
}
function main(): void {
    console.log(invert(true));
    console.log(invert(false));
}
main();
"#);
}
```

- [ ] **Step 2: Run tests — expect FAIL**

- [ ] **Step 3: Add `Expr::Unary` to expression dispatcher**

In `crates/tyrus_codegen/src/convert/expr/mod.rs`, find the match on `Expr` variants and add:
```rust
Expr::Unary(unary) => self.convert_unary_expr(unary),
```

- [ ] **Step 4: Implement `convert_unary_expr` in `expr/misc.rs`**

```rust
pub(crate) fn convert_unary_expr(&self, expr: &UnaryExpr) -> proc_macro2::TokenStream {
    let arg = self.convert_expr(&expr.arg);
    match expr.op {
        UnaryOp::Minus => quote::quote! { -#arg },
        UnaryOp::Plus => arg,  // +x is identity in JS for numbers
        UnaryOp::Bang => quote::quote! { !#arg },
        UnaryOp::TypeOf => quote::quote! { compile_error!("Tyrus: typeof not supported") },
        _ => quote::quote! { compile_error!("Tyrus: unsupported unary operator") },
    }
}
```

- [ ] **Step 5: Run tests — expect PASS**

Run: `cargo nextest run --workspace`

- [ ] **Step 6: Commit**

```bash
git commit -m "fix(codegen): add unary expression support (!, -, +)"
```

---

### Task 7: Fix `trim()` return type (`&str` → `String`)

**Context:** `.trim()` returns `&str` in Rust, but the generated code expects `String`. This causes compilation failure in string operations.

**Files:**
- Test: `tests/src/equivalence/strings.rs`
- Modify: `crates/tyrus_codegen/src/stdlib/string.rs`

- [ ] **Step 1: Write equivalence test**

Create `tests/src/equivalence/strings.rs`:
```rust
use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_string_trim() {
    assert_output_equivalent(r#"
function main(): void {
    const s: string = "  hello  ";
    console.log(s.trim());
}
main();
"#);
}
```

- [ ] **Step 2: Register module**

Add `mod strings;` to `tests/src/equivalence/mod.rs`.

- [ ] **Step 3: Fix trim() in stdlib/string.rs**

Change:
```rust
"trim" => Some(quote! { #obj_tokens.trim() })
```
To:
```rust
"trim" => Some(quote! { #obj_tokens.trim().to_string() })
```

- [ ] **Step 4: Run tests — expect PASS**

- [ ] **Step 5: Commit**

```bash
git commit -m "fix(stdlib): trim() returns String instead of &str"
```

---

## Chunk 3: Basic Equivalence — Arithmetic & Console

### Task 8: Verify basic arithmetic equivalence

**Files:**
- Test: `tests/src/equivalence/basic.rs`

- [ ] **Step 1: Write comprehensive arithmetic tests**

```rust
#[test]
fn test_equivalence_arithmetic_operations() {
    assert_output_equivalent(r#"
function main(): void {
    console.log(2 + 3);
    console.log(10 - 4);
    console.log(3 * 7);
    console.log(15 / 4);
    console.log(17 % 5);
}
main();
"#);
}

#[test]
fn test_equivalence_comparison_operators() {
    assert_output_equivalent(r#"
function main(): void {
    console.log(5 > 3);
    console.log(2 < 1);
    console.log(3 >= 3);
    console.log(4 <= 5);
    console.log(3 === 3);
    console.log(3 !== 4);
}
main();
"#);
}

#[test]
fn test_equivalence_string_concatenation() {
    assert_output_equivalent(r#"
function main(): void {
    const greeting: string = "Hello" + " " + "World";
    console.log(greeting);
}
main();
"#);
}

#[test]
fn test_equivalence_if_else() {
    assert_output_equivalent(r#"
function classify(x: number): string {
    if (x > 0) {
        return "positive";
    } else if (x < 0) {
        return "negative";
    } else {
        return "zero";
    }
}
function main(): void {
    console.log(classify(5));
    console.log(classify(-3));
    console.log(classify(0));
}
main();
"#);
}

#[test]
fn test_equivalence_while_loop() {
    assert_output_equivalent(r#"
function sum(n: number): number {
    let total: number = 0;
    let i: number = 1;
    while (i <= n) {
        total = total + i;
        i = i + 1;
    }
    return total;
}
function main(): void {
    console.log(sum(5));
    console.log(sum(10));
}
main();
"#);
}
```

- [ ] **Step 2: Run all tests**

Run: `cargo test -p integration_tests equivalence`

Debug any output format differences (e.g., `5` vs `5.0`, `true` vs `True`). Fix as needed.

- [ ] **Step 3: Commit**

```bash
git commit -m "test(equivalence): verify basic arithmetic and control flow"
```

---

### Task 9: Verify console.log formatting equivalence

**Files:**
- Test: `tests/src/equivalence/console.rs`

**Important:** Node's `console.log(5.0)` prints `5`, but Rust's `println!("{}", 5f64)` prints `5`. These should match. But `console.log(5.5)` prints `5.5` and `println!("{}", 5.5f64)` also prints `5.5`. Check edge cases.

- [ ] **Step 1: Create console equivalence tests**

`tests/src/equivalence/console.rs`:
```rust
use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_console_log_number() {
    assert_output_equivalent(r#"
function main(): void {
    console.log(42);
}
main();
"#);
}

#[test]
fn test_equivalence_console_log_string() {
    assert_output_equivalent(r#"
function main(): void {
    console.log("hello world");
}
main();
"#);
}

#[test]
fn test_equivalence_console_log_boolean() {
    assert_output_equivalent(r#"
function main(): void {
    console.log(true);
    console.log(false);
}
main();
"#);
}

#[test]
fn test_equivalence_console_log_multiple_args() {
    assert_output_equivalent(r#"
function main(): void {
    console.log("result:", 42);
}
main();
"#);
}
```

- [ ] **Step 2: Register module** — Add `mod console;` to equivalence/mod.rs

- [ ] **Step 3: Run tests and debug format differences**

If Node prints `42` but Rust prints `42` → match.
If Node prints `true` but Rust prints `true` → match.
If Node prints `result: 42` but Rust prints `result: 42` → match.

Fix any mismatches in `crates/tyrus_codegen/src/stdlib/console.rs`.

- [ ] **Step 4: Commit**

```bash
git commit -m "test(equivalence): verify console.log output formatting"
```

---

## Chunk 4: String Methods Equivalence

### Task 10: Verify existing string methods + add missing ones

**Files:**
- Test: `tests/src/equivalence/strings.rs`
- Modify: `crates/tyrus_codegen/src/stdlib/string.rs`

- [ ] **Step 1: Write tests for EXISTING string methods**

Add to `tests/src/equivalence/strings.rs`:
```rust
#[test]
fn test_equivalence_string_to_upper_case() {
    assert_output_equivalent(r#"
function main(): void {
    const s: string = "hello";
    console.log(s.toUpperCase());
}
main();
"#);
}

#[test]
fn test_equivalence_string_to_lower_case() {
    assert_output_equivalent(r#"
function main(): void {
    const s: string = "HELLO";
    console.log(s.toLowerCase());
}
main();
"#);
}

#[test]
fn test_equivalence_string_includes() {
    assert_output_equivalent(r#"
function main(): void {
    const s: string = "hello world";
    console.log(s.includes("world"));
    console.log(s.includes("xyz"));
}
main();
"#);
}

#[test]
fn test_equivalence_string_replace() {
    assert_output_equivalent(r#"
function main(): void {
    const s: string = "hello world";
    console.log(s.replace("world", "rust"));
}
main();
"#);
}

#[test]
fn test_equivalence_string_split() {
    assert_output_equivalent(r#"
function main(): void {
    const s: string = "a,b,c";
    const parts: string[] = s.split(",");
    console.log(parts.length);
}
main();
"#);
}
```

- [ ] **Step 2: Run tests — fix any failures**

- [ ] **Step 3: Add `startsWith` and `endsWith` to stdlib/string.rs**

```rust
"startsWith" => {
    if let Some(arg) = args.first() {
        let val = gen.convert_expr_or_spread(arg);
        Some(quote! { #obj_tokens.starts_with(&#val as &str) })
    } else {
        None
    }
}
"endsWith" => {
    if let Some(arg) = args.first() {
        let val = gen.convert_expr_or_spread(arg);
        Some(quote! { #obj_tokens.ends_with(&#val as &str) })
    } else {
        None
    }
}
```

- [ ] **Step 4: Write equivalence tests for new methods**

```rust
#[test]
fn test_equivalence_string_starts_with() {
    assert_output_equivalent(r#"
function main(): void {
    const s: string = "hello world";
    console.log(s.startsWith("hello"));
    console.log(s.startsWith("world"));
}
main();
"#);
}

#[test]
fn test_equivalence_string_ends_with() {
    assert_output_equivalent(r#"
function main(): void {
    const s: string = "hello world";
    console.log(s.endsWith("world"));
    console.log(s.endsWith("hello"));
}
main();
"#);
}
```

- [ ] **Step 5: Run tests — expect PASS**

- [ ] **Step 6: Commit**

```bash
git commit -m "feat(stdlib): add startsWith/endsWith + equivalence tests for string methods"
```

---

## Chunk 5: Array Methods Equivalence

### Task 11: Verify existing array methods + add missing ones

**Files:**
- Test: `tests/src/equivalence/arrays.rs`
- Modify: `crates/tyrus_codegen/src/stdlib/array.rs`

- [ ] **Step 1: Create array equivalence tests**

`tests/src/equivalence/arrays.rs`:
```rust
use crate::helpers::assert_output_equivalent;

#[test]
fn test_equivalence_array_map() {
    assert_output_equivalent(r#"
function main(): void {
    const nums: number[] = [1, 2, 3, 4, 5];
    const doubled: number[] = nums.map((n: number): number => n * 2);
    console.log(doubled.join(","));
}
main();
"#);
}

#[test]
fn test_equivalence_array_filter() {
    assert_output_equivalent(r#"
function main(): void {
    const nums: number[] = [1, 2, 3, 4, 5, 6];
    const evens: number[] = nums.filter((n: number): boolean => n % 2 === 0);
    console.log(evens.join(","));
}
main();
"#);
}

#[test]
fn test_equivalence_array_for_each() {
    assert_output_equivalent(r#"
function main(): void {
    const nums: number[] = [10, 20, 30];
    nums.forEach((n: number): void => {
        console.log(n);
    });
}
main();
"#);
}
```

- [ ] **Step 2: Register module** — Add `mod arrays;` to equivalence/mod.rs

- [ ] **Step 3: Run tests — debug any output format differences**

Array output in JS vs Rust may differ. `console.log([1,2,3])` in JS prints `1,2,3` but in Rust a Vec prints differently. Use `.join(",")` in fixtures to normalize output.

- [ ] **Step 4: Add `includes()` to stdlib/array.rs**

```rust
"includes" => {
    if let Some(arg) = args.first() {
        let val = gen.convert_expr_or_spread(arg);
        Some(quote! { #obj_tokens.contains(&#val) })
    } else {
        None
    }
}
```

- [ ] **Step 5: Write test for includes**

```rust
#[test]
fn test_equivalence_array_includes() {
    assert_output_equivalent(r#"
function main(): void {
    const nums: number[] = [1, 2, 3, 4, 5];
    console.log(nums.includes(3));
    console.log(nums.includes(9));
}
main();
"#);
}
```

- [ ] **Step 6: Run tests — expect PASS**

- [ ] **Step 7: Commit**

```bash
git commit -m "feat(stdlib): add array.includes + equivalence tests for array methods"
```

---

## Chunk 6: Documentation Update + Final Verification

### Task 12: Update all project documentation

**Files:**
- Modify: `ROADMAP.md` — Add Milestone 13A
- Modify: `CLAUDE.md` — Update test count and capabilities
- Modify: `docs/ARCHITECTURE.md` — Add equivalence testing section
- Modify: `docs/specs/GRAMMAR.md` — If any new constructs were added

- [ ] **Step 1: Update ROADMAP.md**

Add after Milestone 12:
```markdown
### 🔬 Milestone 13A: Semantic Equivalence + Basic Coverage

- [x] **Equivalence Testing Infrastructure:** `assert_output_equivalent()` — runs TS (Node.js) and generated Rust, compares stdout
- [x] **Bug Fix — const/let:** `const` → `let` (immutable), `let` → `let mut` (mutable)
- [x] **Bug Fix — Unary ops:** Support for `!`, `-`, `+` operators
- [x] **Bug Fix — trim():** Returns `String` instead of `&str`
- [x] **String Methods:** `startsWith`, `endsWith` added
- [x] **Array Methods:** `includes` added
- [x] **Equivalence Tests:** N tests verifying TS↔Rust output identity
```

- [ ] **Step 2: Update CLAUDE.md test count**

- [ ] **Step 3: Commit**

```bash
git commit -m "docs(project): update documentation for Milestone 13A"
```

### Task 13: Final verification

- [ ] **Step 1: Run full test suite**

```bash
cargo nextest run --workspace
```

All tests must pass (old + new).

- [ ] **Step 2: Run clippy**

```bash
cargo clippy --workspace
```

Zero warnings.

- [ ] **Step 3: Run fmt check**

```bash
cargo fmt -- --check
```

- [ ] **Step 4: Verify demo still works**

```bash
cargo run --bin tyrus -- build examples/real_world_demo/src --output examples/real_world_demo/output && cd examples/real_world_demo/output && cargo check
```

---

## Summary

| Chunk | Tasks | What it proves |
|-------|-------|---------------|
| 1: Infrastructure | Tasks 1-4 | `assert_output_equivalent()` works end-to-end |
| 2: Bug Fixes | Tasks 5-7 | const/let, unary ops, trim() all fixed |
| 3: Basic Equiv | Tasks 8-9 | Arithmetic + console.log output matches TS↔Rust |
| 4: String Equiv | Task 10 | 7 string methods verified + 2 new ones added |
| 5: Array Equiv | Task 11 | 4 array methods verified + includes added |
| 6: Docs + Final | Tasks 12-13 | Everything green, docs updated |

**Expected new test count:** ~15-20 equivalence tests on top of existing 86.
**Expected outcome:** Every basic transpilation pattern is PROVEN to produce identical output to TypeScript.
