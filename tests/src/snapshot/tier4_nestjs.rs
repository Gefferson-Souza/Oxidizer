use crate::helpers::transpile_fixture;

#[test]
fn test_snapshot_injectable_service() {
    let rust = transpile_fixture("tier4/injectable_service");
    insta::assert_snapshot!(rust);
}

#[test]
fn test_snapshot_controller() {
    let rust = transpile_fixture("tier4/controller");
    insta::assert_snapshot!(rust);
}
