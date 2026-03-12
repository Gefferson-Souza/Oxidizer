use crate::helpers::transpile_fixture;
use tyrus_test_utils::assert_rust_compiles;

/// Batch compile ALL tier1 fixtures in a single test.
/// This is intentionally a single test to minimize cargo check invocations.
#[test]
fn test_tier1_compiles() {
    let fixtures = [
        "tier1/variables",
        "tier1/math_ops",
        "tier1/string_ops",
        "tier1/functions",
        "tier1/control_flow",
        "tier1/console",
    ];

    for fixture in &fixtures {
        let rust = transpile_fixture(fixture);
        assert_rust_compiles(&rust);
    }
}
