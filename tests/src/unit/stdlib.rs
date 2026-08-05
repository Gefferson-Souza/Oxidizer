use crate::helpers::transpile;

// ── Console ───────────────────────────────────────────────────────────

#[test]
fn test_console_log_number() {
    let rust = transpile("function f(): void { console.log(42); }");
    assert!(rust.contains("println!"), "Expected 'println!' in: {rust}");
    assert!(rust.contains("42f64"), "Expected '42f64' in: {rust}");
}

#[test]
fn test_console_log_string() {
    let rust = transpile(r#"function f(): void { console.log("hello"); }"#);
    assert!(rust.contains("println!"), "Expected 'println!' in: {rust}");
}

#[test]
fn test_console_error() {
    let rust = transpile(r#"function f(): void { console.error("error"); }"#);
    assert!(
        rust.contains("eprintln!"),
        "Expected 'eprintln!' in: {rust}"
    );
}

#[test]
fn test_console_log_variable() {
    let rust = transpile("function f(x: number): void { console.log(x); }");
    assert!(rust.contains("println!"), "Expected 'println!' in: {rust}");
    assert!(rust.contains('x'), "Expected 'x' in: {rust}");
}
