use crate::helpers::transpile_fixture;

#[test]
fn test_snapshot_variables() {
    let rust = transpile_fixture("tier1/variables");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_math_ops() {
    let rust = transpile_fixture("tier1/math_ops");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_string_ops() {
    let rust = transpile_fixture("tier1/string_ops");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_functions() {
    let rust = transpile_fixture("tier1/functions");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_control_flow() {
    let rust = transpile_fixture("tier1/control_flow");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_console() {
    let rust = transpile_fixture("tier1/console");
    insta::assert_snapshot!(rust);
}
