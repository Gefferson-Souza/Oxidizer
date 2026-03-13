use crate::helpers::transpile;

// ── Variable Declarations ──────────────────────────────────────────────

#[test]
fn test_const_string_declaration() {
    let rust = transpile(r#"function f(): void { const name: string = "Alice"; }"#);
    assert!(
        rust.contains("let name") && !rust.contains("let mut name"),
        "Expected immutable 'let name' (not 'let mut') in: {rust}"
    );
    assert!(
        rust.contains("String::from"),
        "Expected 'String::from' in: {rust}"
    );
}

#[test]
fn test_const_number_declaration() {
    let rust = transpile("function f(): void { const x: number = 42; }");
    assert!(
        rust.contains("let x") && !rust.contains("let mut x"),
        "Expected immutable 'let x' (not 'let mut') in: {rust}"
    );
    assert!(rust.contains("42f64"), "Expected '42f64' in: {rust}");
}

#[test]
fn test_const_boolean_declaration() {
    let rust = transpile("function f(): void { const flag: boolean = true; }");
    assert!(
        rust.contains("let flag") && !rust.contains("let mut flag"),
        "Expected immutable 'let flag' (not 'let mut') in: {rust}"
    );
    assert!(rust.contains("true"), "Expected 'true' in: {rust}");
}

#[test]
fn test_let_declaration_is_mutable() {
    let rust = transpile("function f(): void { let count: number = 0; }");
    assert!(
        rust.contains("let mut count"),
        "Expected 'let mut count' in: {rust}"
    );
    assert!(rust.contains("0f64"), "Expected '0f64' in: {rust}");
}

// ── Binary Expressions ────────────────────────────────────────────────

#[test]
fn test_addition() {
    let rust = transpile("function add(a: number, b: number): number { return a + b; }");
    assert!(rust.contains("a + b"), "Expected 'a + b' in: {rust}");
}

#[test]
fn test_subtraction() {
    let rust = transpile("function sub(a: number, b: number): number { return a - b; }");
    assert!(rust.contains("a - b"), "Expected 'a - b' in: {rust}");
}

#[test]
fn test_multiplication() {
    let rust = transpile("function mul(a: number, b: number): number { return a * b; }");
    assert!(rust.contains("a * b"), "Expected 'a * b' in: {rust}");
}

#[test]
fn test_division() {
    let rust = transpile("function div(a: number, b: number): number { return a / b; }");
    assert!(rust.contains("a / b"), "Expected 'a / b' in: {rust}");
}

// ── String Operations ─────────────────────────────────────────────────

#[test]
fn test_string_template_literal() {
    let rust = transpile(r#"function greet(name: string): string { return `Hello, ${name}!`; }"#);
    assert!(rust.contains("format!"), "Expected 'format!' in: {rust}");
    assert!(
        rust.contains("Hello, {}!"),
        "Expected 'Hello, {{}}!' in: {rust}"
    );
    assert!(rust.contains("name"), "Expected 'name' in: {rust}");
}

// ── String Union Enums ───────────────────────────────────────────────────

#[test]
fn test_string_union_generates_enum() {
    let rust = transpile(r#"type Status = "active" | "inactive" | "pending";"#);
    assert!(
        rust.contains("enum Status"),
        "Expected 'enum Status' in: {rust}"
    );
    assert!(rust.contains("Active"), "Expected 'Active' in: {rust}");
    assert!(rust.contains("Inactive"), "Expected 'Inactive' in: {rust}");
    assert!(rust.contains("Pending"), "Expected 'Pending' in: {rust}");
}
