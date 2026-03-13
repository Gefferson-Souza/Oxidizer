use crate::helpers::transpile_fixture;
use tyrus_test_utils::assert_rust_compiles;

#[test]
fn test_tier3_generics_compiles() {
    let rust = transpile_fixture("tier3/generics");
    assert_rust_compiles(&rust);
}

#[test]
#[ignore = "optional chaining on Option<T> fields creates double-Option — needs type inference (Phase 6)"]
fn test_tier3_optional_chaining_compiles() {
    let rust = transpile_fixture("tier3/optional_chaining");
    assert_rust_compiles(&rust);
}

#[test]
fn test_tier3_destructuring_compiles() {
    let rust = transpile_fixture("tier3/destructuring");
    assert_rust_compiles(&rust);
}

#[test]
fn test_tier3_advanced_methods_compiles() {
    let rust = transpile_fixture("tier3/advanced_methods");
    assert_rust_compiles(&rust);
}

#[test]
fn test_tier3_string_methods_compiles() {
    let rust = transpile_fixture("tier3/string_methods");
    assert_rust_compiles(&rust);
}

#[test]
fn test_tier3_math_stdlib_compiles() {
    let rust = transpile_fixture("tier3/math_stdlib");
    assert_rust_compiles(&rust);
}

#[test]
fn test_tier3_ternary_compiles() {
    let rust = transpile_fixture("tier3/ternary");
    assert_rust_compiles(&rust);
}
