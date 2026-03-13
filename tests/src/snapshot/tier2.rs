use crate::helpers::transpile_fixture;

#[test]
fn test_snapshot_interfaces() {
    let rust = transpile_fixture("tier2/interfaces");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_type_aliases() {
    let rust = transpile_fixture("tier2/type_aliases");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_arrays() {
    let rust = transpile_fixture("tier2/arrays");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_classes() {
    let rust = transpile_fixture("tier2/classes");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_async_await() {
    let rust = transpile_fixture("tier2/async_await");
    insta::assert_snapshot!(rust);
}
