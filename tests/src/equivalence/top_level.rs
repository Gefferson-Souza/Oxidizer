use crate::helpers::{assert_output_equivalent, transpile};

#[test]
fn test_equivalence_top_level_const() {
    assert_output_equivalent(
        r#"
const greeting: string = "Hello World";
const count: number = 42;
const active: boolean = true;
console.log(greeting);
console.log(count);
console.log(active);
"#,
    );
}

#[test]
fn test_equivalence_top_level_let() {
    assert_output_equivalent(
        r"
let counter: number = 0;
counter = counter + 1;
counter = counter + 1;
console.log(counter);
",
    );
}

#[test]
fn test_equivalence_top_level_mixed() {
    assert_output_equivalent(
        r#"
function greet(name: string): string {
    return "Hello, " + name;
}
const message: string = greet("World");
console.log(message);
"#,
    );
}

/// Regression test for #186 (UAT-C1): `ExprStmt` at module level was silently
/// dropped because `process_module_item` used `stmt.visit_with(self)`
/// (children-only traversal) instead of `self.visit_stmt(stmt)`.
#[test]
fn test_equivalence_top_level_bare_expr_stmt() {
    assert_output_equivalent(
        r"
console.log(1 + 1);
",
    );
}

/// Regression test for #186: top-level if/else statement (non-decl Stmt
/// variant) must reach `main_body`.
#[test]
fn test_equivalence_top_level_if_stmt() {
    assert_output_equivalent(
        r#"
const score: number = 85;
if (score >= 70) {
    console.log("pass");
} else {
    console.log("fail");
}
"#,
    );
}

/// Regression test for #186: top-level for-loop must reach `main_body`.
#[test]
fn test_equivalence_top_level_for_loop() {
    assert_output_equivalent(
        r"
let total: number = 0;
for (const n of [1, 2, 3, 4, 5]) {
    total = total + n;
}
console.log(total);
",
    );
}

/// Regression test for #186: full UAT quick.ts scenario — interface +
/// multiple function decls + top-level object construction + console.log.
#[test]
fn test_equivalence_top_level_uat_quick_scenario() {
    assert_output_equivalent(
        r#"
interface User {
    id: number;
    name: string;
    active: boolean;
}

function isAdult(age: number): boolean {
    return age >= 18;
}

const alice: User = {
    id: 1,
    name: "Alice",
    active: true,
};

console.log(alice.name);
console.log("isAdult(20):", isAdult(20));
"#,
    );
}

/// R7-incidental verification: the `fn main()` wrapper is emitted via
/// `quote!` (not `push_str`). Verifies generated code structure.
#[test]
fn test_top_level_main_wrapper_is_present() {
    let rust_code = transpile(
        r#"
console.log("hello");
"#,
    );
    assert!(
        rust_code.contains("fn main()"),
        "expected fn main() wrapper, got:\n{rust_code}"
    );
    assert!(
        rust_code.contains("println"),
        "expected println! emission, got:\n{rust_code}"
    );
}
