use crate::helpers::transpile;

// ── Type Mappings (TS → Rust) ─────────────────────────────────────────

#[test]
fn test_number_maps_to_f64() {
    let rust = transpile("function f(x: number): number { return x; }");
    assert!(rust.contains("x: f64"), "Expected 'x: f64' in: {rust}");
    assert!(rust.contains("-> f64"), "Expected '-> f64' in: {rust}");
}

#[test]
fn test_string_maps_to_string() {
    let rust = transpile("function f(x: string): string { return x; }");
    assert!(
        rust.contains("x: String"),
        "Expected 'x: String' in: {rust}"
    );
    assert!(
        rust.contains("-> String"),
        "Expected '-> String' in: {rust}"
    );
}

#[test]
fn test_boolean_maps_to_bool() {
    let rust = transpile("function f(x: boolean): boolean { return x; }");
    assert!(rust.contains("x: bool"), "Expected 'x: bool' in: {rust}");
    assert!(rust.contains("-> bool"), "Expected '-> bool' in: {rust}");
}

#[test]
fn test_void_maps_to_unit() {
    let rust = transpile("function f(): void { }");
    assert!(rust.contains("-> ()"), "Expected '-> ()' in: {rust}");
}

#[test]
fn test_number_literal_suffix() {
    let rust = transpile("function f(): void { const x: number = 99; }");
    assert!(rust.contains("99f64"), "Expected '99f64' in: {rust}");
}

#[test]
fn test_string_literal_uses_string_from() {
    let rust = transpile(r#"function f(): void { const s: string = "hi"; }"#);
    assert!(
        rust.contains("String::from"),
        "Expected 'String::from' in: {rust}"
    );
}

// ── Interface / Struct Mappings ──────────────────────────────────────────

#[test]
fn test_interface_generates_struct() {
    let rust = transpile("interface User { name: string; age: number; }");
    assert!(
        rust.contains("struct User"),
        "Expected 'struct User' in: {rust}"
    );
    assert!(
        rust.contains("name: String"),
        "Expected 'name: String' in: {rust}"
    );
    assert!(rust.contains("age: f64"), "Expected 'age: f64' in: {rust}");
}

#[test]
fn test_interface_has_serde_derive() {
    let rust = transpile("interface User { name: string; }");
    assert!(
        rust.contains("Serialize"),
        "Expected 'Serialize' in: {rust}"
    );
    assert!(
        rust.contains("Deserialize"),
        "Expected 'Deserialize' in: {rust}"
    );
}

#[test]
fn test_optional_field() {
    let rust = transpile("interface Config { debug?: boolean; }");
    assert!(
        rust.contains("Option<bool>"),
        "Expected 'Option<bool>' in: {rust}"
    );
}
