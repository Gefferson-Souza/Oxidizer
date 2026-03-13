use crate::helpers::transpile_fixture;

#[test]
fn test_snapshot_generics() {
    let rust = transpile_fixture("tier3/generics");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_optional_chaining() {
    let rust = transpile_fixture("tier3/optional_chaining");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_destructuring() {
    let rust = transpile_fixture("tier3/destructuring");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_advanced_methods() {
    let rust = transpile_fixture("tier3/advanced_methods");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_string_methods() {
    let rust = transpile_fixture("tier3/string_methods");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_math_stdlib() {
    let rust = transpile_fixture("tier3/math_stdlib");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_ternary() {
    let rust = transpile_fixture("tier3/ternary");
    insta::assert_snapshot!(rust);
}
