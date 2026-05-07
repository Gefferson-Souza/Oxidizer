use crate::helpers::transpile_fixture;

/// Snapshot of the full UAT quick.ts scenario — interface + multiple
/// function decls + top-level object construction + console.log block.
/// Verifies #186 fix produces a complete `fn main()` wrapper with all
/// top-level statements lifted in source order.
#[test]
fn test_snapshot_top_level_uat_quick() {
    let rust = transpile_fixture("uat/quick");
    insta::assert_snapshot!(rust);
}
