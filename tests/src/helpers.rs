use std::path::Path;
use std::time::Duration;

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

/// Asserts that transpiled Rust produces identical stdout to the original TypeScript.
///
/// The TypeScript code supports:
/// - Function declarations (transpiled as Rust functions)
/// - Top-level statements (wrapped in `fn main()` automatically)
/// - `console.log()` for observable output
pub fn assert_output_equivalent(ts_code: &str) {
    // 1. Run TypeScript with Node.js
    let ts_output = tyrus_test_utils::run_node(ts_code);

    // 2. Transpile TS → Rust
    let rust_code = transpile(ts_code);

    // 3. Ensure there's an entry point
    let rust_binary = if rust_code.contains("fn main()") {
        rust_code.clone()
    } else {
        format!("{}\nfn main() {{}}", rust_code)
    };

    // 4. Compile + run Rust
    let rust_output = tyrus_test_utils::compile_and_run_rust(&rust_binary);

    // 5. Compare outputs
    assert_eq!(
        ts_output.trim(),
        rust_output.trim(),
        "\n╔══════════════════════════════════════════╗\n\
         ║  SEMANTIC EQUIVALENCE FAILURE             ║\n\
         ╚══════════════════════════════════════════╝\n\n\
         TypeScript output: {:?}\n\
         Rust output:       {:?}\n\n\
         Generated Rust code:\n{}\n",
        ts_output.trim(),
        rust_output.trim(),
        rust_code
    );
}

/// Same contract as [`assert_output_equivalent`] but bounds the Rust binary's
/// execution wall-clock time. Use for tests that exercise code paths prone
/// to deadlock (e.g., mutex re-entrance regressions).
///
/// On timeout, the child process is killed and the test panics with a message
/// that names the timeout — preventing nextest from hanging indefinitely.
pub fn assert_output_equivalent_with_timeout(ts_code: &str, timeout: Duration) {
    let ts_output = tyrus_test_utils::run_node(ts_code);
    let rust_code = transpile(ts_code);

    let rust_binary = if rust_code.contains("fn main()") {
        rust_code.clone()
    } else {
        format!("{}\nfn main() {{}}", rust_code)
    };

    let rust_output = tyrus_test_utils::compile_and_run_rust_with_timeout(&rust_binary, timeout);

    assert_eq!(
        ts_output.trim(),
        rust_output.trim(),
        "\n╔══════════════════════════════════════════╗\n\
         ║  SEMANTIC EQUIVALENCE FAILURE             ║\n\
         ╚══════════════════════════════════════════╝\n\n\
         TypeScript output: {:?}\n\
         Rust output:       {:?}\n\n\
         Generated Rust code:\n{}\n",
        ts_output.trim(),
        rust_output.trim(),
        rust_code
    );
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
