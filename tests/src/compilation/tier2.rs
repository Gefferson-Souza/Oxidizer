use crate::helpers::transpile_fixture;
use tyrus_test_utils::assert_rust_compiles;

/// Batch compile ALL tier2 fixtures in a single test.
/// This is intentionally a single test to minimize cargo check invocations.
#[test]
fn test_tier2_compiles() {
    let fixtures = [
        "tier2/interfaces",
        "tier2/type_aliases",
        "tier2/arrays",
        "tier2/classes",
        "tier2/async_await",
    ];

    for fixture in &fixtures {
        let rust = transpile_fixture(fixture);
        assert_rust_compiles(&rust);
    }
}
