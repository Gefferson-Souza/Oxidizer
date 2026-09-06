use crate::helpers::assert_output_equivalent;

/// Type assertion `as Type` is a no-op in TypeScript at runtime.
/// The transpiler should produce valid Rust that prints the same value
/// regardless of whether it handles the `as` cast syntax or simply drops it.
#[test]
fn test_equivalence_type_assertion_noop() {
    assert_output_equivalent(
        r#"
function getUser(): string {
    const data: string = "Alice";
    return data;
}
function run(): void {
    const user: string = getUser();
    console.log(user);
}
run();
"#,
    );
}

/// Tests a function that maps numeric values to string names — the same
/// pattern that would arise when using a numeric enum's variant values
/// (e.g., `Direction.Up === 0`). Using plain numeric literals here keeps
/// the fixture inside the Oxidizable Standard while validating the
/// conditional-dispatch logic needed for enum support.
#[test]
fn test_equivalence_numeric_enum_values() {
    assert_output_equivalent(
        r#"
function dirName(d: number): string {
    if (d === 0) { return "Up"; }
    if (d === 1) { return "Down"; }
    if (d === 2) { return "Left"; }
    return "Right";
}
function run(): void {
    console.log(dirName(0));
    console.log(dirName(1));
    console.log(dirName(3));
}
run();
"#,
    );
}
