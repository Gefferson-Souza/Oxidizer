use crate::helpers::transpile;

#[test]
fn test_generic_interface() {
    let rust = transpile("interface Box<T> { value: T; }");
    assert!(
        rust.contains("struct Box"),
        "Expected 'struct Box' in: {rust}"
    );
    assert!(
        rust.contains("<T"),
        "Expected generic parameter '<T' in: {rust}"
    );
}

#[test]
fn test_generic_function() {
    let rust = transpile("function identity<T>(value: T): T { return value; }");
    assert!(
        rust.contains("fn identity"),
        "Expected 'fn identity' in: {rust}"
    );
}

#[test]
fn test_ternary_expression() {
    let rust = transpile("function max(a: number, b: number): number { return a > b ? a : b; }");
    assert!(rust.contains("if a > b"), "Expected 'if a > b' in: {rust}");
}

#[test]
fn test_string_includes_maps_to_contains() {
    let rust =
        transpile("function has(s: string, sub: string): boolean { return s.includes(sub); }");
    assert!(rust.contains("contains"), "Expected 'contains' in: {rust}");
}

#[test]
fn test_to_upper_case() {
    let rust = transpile("function upper(s: string): string { return s.toUpperCase(); }");
    assert!(
        rust.contains("to_uppercase"),
        "Expected 'to_uppercase' in: {rust}"
    );
}

#[test]
fn test_math_max() {
    let rust =
        transpile("function bigger(a: number, b: number): number { return Math.max(a, b); }");
    assert!(rust.contains("max"), "Expected 'max' in: {rust}");
}
