use crate::helpers::transpile;

// ── Function Signatures ───────────────────────────────────────────────

#[test]
fn test_function_with_number_params() {
    let rust = transpile("function add(a: number, b: number): number { return a + b; }");
    assert!(rust.contains("fn add"), "Expected 'fn add' in: {rust}");
    assert!(rust.contains("a: f64"), "Expected 'a: f64' in: {rust}");
    assert!(rust.contains("b: f64"), "Expected 'b: f64' in: {rust}");
    assert!(rust.contains("-> f64"), "Expected '-> f64' in: {rust}");
}

#[test]
fn test_function_with_string_param() {
    let rust = transpile("function greet(name: string): string { return name; }");
    assert!(rust.contains("fn greet"), "Expected 'fn greet' in: {rust}");
    assert!(
        rust.contains("name: String"),
        "Expected 'name: String' in: {rust}"
    );
    assert!(
        rust.contains("-> String"),
        "Expected '-> String' in: {rust}"
    );
}

#[test]
fn test_function_returns_bool() {
    let rust = transpile("function check(x: number): boolean { return x > 0; }");
    assert!(rust.contains("-> bool"), "Expected '-> bool' in: {rust}");
}

#[test]
fn test_function_returns_void() {
    let rust = transpile("function noop(): void { }");
    assert!(rust.contains("-> ()"), "Expected '-> ()' in: {rust}");
}

// ── Control Flow ──────────────────────────────────────────────────────

#[test]
fn test_if_statement() {
    let rust = transpile("function pos(x: number): number { if (x > 0) { return x; } return 0; }");
    assert!(
        rust.contains("if x > 0f64"),
        "Expected 'if x > 0f64' in: {rust}"
    );
}

#[test]
fn test_if_else_statement() {
    let rust = transpile(
        "function sign(x: number): number { if (x > 0) { return x; } else { return 0; } }",
    );
    assert!(
        rust.contains("if x > 0f64"),
        "Expected 'if x > 0f64' in: {rust}"
    );
    assert!(
        rust.contains("} else {"),
        "Expected '}} else {{' in: {rust}"
    );
}

#[test]
fn test_while_loop() {
    let rust = transpile("function f(): void { let i: number = 0; while (i < 10) { i = i + 1; } }");
    assert!(
        rust.contains("while i < 10f64"),
        "Expected 'while i < 10f64' in: {rust}"
    );
}

#[test]
fn test_return_statement() {
    let rust = transpile("function id(x: number): number { return x; }");
    assert!(rust.contains("return x"), "Expected 'return x' in: {rust}");
}
