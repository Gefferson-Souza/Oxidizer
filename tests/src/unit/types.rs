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

// ── Issue #130 — integer-shape JSON contract ──────────────────────────

#[test]
fn test_id_field_emits_int_serde_attr() {
    let rust = transpile("interface Item { id: number; name: string; }");
    assert!(
        rust.contains("__tyrus_int_serde"),
        "Expected integer-shape serde attribute on id field, got: {rust}"
    );
    assert!(
        rust.contains("pub id : f64") || rust.contains("pub id: f64"),
        "Field type must remain f64 (not changed to i64); rust: {rust}"
    );
}

#[test]
fn test_camel_case_id_suffix_triggers_heuristic() {
    let rust = transpile("interface Order { userId: number; total: number; }");
    assert!(
        rust.contains("__tyrus_int_serde"),
        "Expected userId to trigger integer-shape attr; rust: {rust}"
    );
    let after_total = rust
        .split("pub total")
        .nth(1)
        .unwrap_or("")
        .split('}')
        .next()
        .unwrap_or("");
    assert!(
        !after_total.contains("__tyrus_int_serde"),
        "Non-integer-shape field 'total' should not get the attribute"
    );
}

#[test]
fn test_int_serde_helper_module_emitted_when_used() {
    let rust = transpile("interface Item { id: number; }");
    assert!(
        rust.contains("mod __tyrus_int_serde"),
        "Helper module must be emitted at the top when at least one field uses it"
    );
}

#[test]
fn test_int_serde_helper_module_skipped_when_unused() {
    let rust = transpile("interface Plain { name: string; price: number; }");
    assert!(
        !rust.contains("__tyrus_int_serde"),
        "Helper module must be omitted when no field triggers the heuristic"
    );
}
