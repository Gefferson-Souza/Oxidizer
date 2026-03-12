use std::path::Path;

/// Transpile a TypeScript string to Rust code (no compilation).
/// This is the primary test helper — fast, no I/O beyond a temp file.
pub fn transpile(ts_code: &str) -> String {
    let tmp = tempfile::Builder::new()
        .suffix(".ts")
        .tempfile()
        .expect("tmp file");
    std::fs::write(tmp.path(), ts_code).expect("write ts");
    tyrus_orchestrator::build(&tmp.path().to_path_buf().into())
        .unwrap_or_else(|e| panic!("Transpilation failed: {e}"))
}

/// Transpile a fixture file by name (e.g., "tier1/variables").
pub fn transpile_fixture(fixture: &str) -> String {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(format!("{fixture}.ts"));
    assert!(
        fixture_path.exists(),
        "Fixture not found: {}",
        fixture_path.display()
    );
    tyrus_orchestrator::build(&fixture_path.into())
        .unwrap_or_else(|e| panic!("Transpilation failed for {fixture}: {e}"))
}
