use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
}

fn tyrus_cmd() -> Command {
    // Why allowed: `assert_cmd` 2.x deprecates `Command::cargo_bin` but the
    // suggested replacement (`cargo_bin_str`) is not yet stable on our pinned
    // version. Test-only call; migration tracked alongside `assert_cmd` bump.
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("tyrus").expect("tyrus binary not found");
    cmd.current_dir(workspace_root());
    cmd
}

#[test]
fn test_cli_help_shows_all_commands() {
    tyrus_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("compile"))
        .stdout(predicate::str::contains("run"));
}

#[test]
fn test_cli_version() {
    tyrus_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tyrus"));
}

#[test]
fn test_cli_check_valid_file() {
    tyrus_cmd()
        .args(["--quiet", "check", "tests/fixtures/tier1/functions.ts"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Oxidizable"));
}

#[test]
fn test_cli_check_invalid_file() {
    tyrus_cmd()
        .args(["--quiet", "check", "nonexistent.ts"])
        .assert()
        .failure();
}

#[test]
fn test_cli_build_single_file() {
    tyrus_cmd()
        .args(["--quiet", "build", "tests/fixtures/tier1/functions.ts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fn "));
}

#[test]
fn test_cli_check_with_json_flag() {
    // Per plan D4: JSON envelope is a versioned object, not a bare array.
    tyrus_cmd()
        .args([
            "--quiet",
            "check",
            "tests/fixtures/tier1/functions.ts",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schemaVersion\""))
        .stdout(predicate::str::contains("\"status\": \"ok\""));
}

/// Plan D5 — exit code is the primary signal. A file with hard errors
/// must produce a non-zero exit even though it parses successfully.
#[test]
fn test_cli_check_returns_nonzero_on_lint_error() {
    tyrus_cmd()
        .args(["--quiet", "check", "tests/fixtures/invalid/var_decl.ts"])
        .assert()
        .failure();
}

/// Plan D4 — `--json` must include the errors in the envelope. Pre-PR,
/// the JSON output was an empty array; post-PR it carries `errors`.
#[test]
fn test_cli_check_json_includes_errors() {
    let output = tyrus_cmd()
        .args([
            "--quiet",
            "check",
            "tests/fixtures/invalid/var_decl.ts",
            "--json",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\"status\": \"error\""),
        "expected status:error in JSON envelope, got:\n{stdout}"
    );
    assert!(
        stdout.contains("tyrus::lint::no_var"),
        "expected no_var diagnostic code in JSON envelope, got:\n{stdout}"
    );
    // Even with --json, exit code remains the primary signal per D5.
    assert!(
        !output.status.success(),
        "expected non-zero exit code with --json, got success"
    );
}

/// Plan D5 — `--json` envelope contract holds even on pre-analysis
/// failures (file not found, parse error). Tooling can always parse
/// stdout, regardless of which pipeline stage failed.
#[test]
fn test_cli_check_json_envelope_on_missing_file() {
    let output = tyrus_cmd()
        .args([
            "--quiet",
            "check",
            "tests/fixtures/__nonexistent.ts",
            "--json",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("envelope must be valid JSON even on IO error");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["status"], "error");
    assert!(
        json["errors"].as_array().is_some_and(|a| !a.is_empty()),
        "envelope must include the IO error"
    );
    // Rule 14: every error entry carries the stable identifiers.
    assert_eq!(json["errors"][0]["errorCode"], "TYRUS-E3001");
    assert_eq!(json["errors"][0]["category"], "io");
    assert!(!output.status.success());
}

/// Plan D5 — `status` reflects hard errors only; `--strict` exit code
/// can be non-zero even when `status` is `"ok"`. Documents the contract
/// so consumers know to consult exit code as the primary signal.
#[test]
fn test_cli_check_json_status_excludes_strict_promotion() {
    let output = tyrus_cmd()
        .args([
            "--quiet",
            "check",
            "tests/fixtures/invalid/uses_settimeout.ts",
            "--json",
            "--strict",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("envelope must be valid JSON");
    assert_eq!(json["status"], "ok", "status reflects hard errors only");
    assert!(
        json["diagnostics"]
            .as_array()
            .is_some_and(|a| !a.is_empty()),
        "diagnostics must carry the setTimeout entry"
    );
    assert!(
        !output.status.success(),
        "--strict still exits non-zero per D5"
    );
}

/// Plan D3 — `--strict` flag promotes soft diagnostics (e.g., unsupported
/// API uses like setTimeout) to fatal. Default mode keeps them advisory.
#[test]
fn test_cli_check_strict_promotes_diagnostics() {
    // Default: setTimeout is a Diagnostic, not an error → exit 0.
    tyrus_cmd()
        .args([
            "--quiet",
            "check",
            "tests/fixtures/invalid/uses_settimeout.ts",
        ])
        .assert()
        .success();

    // --strict: same file fails because diagnostics are promoted.
    tyrus_cmd()
        .args([
            "--quiet",
            "check",
            "tests/fixtures/invalid/uses_settimeout.ts",
            "--strict",
        ])
        .assert()
        .failure();
}

/// Plan D4 — envelope is versioned. External tooling consumers can pin
/// against `schemaVersion` to detect breaking changes.
#[test]
fn test_cli_check_json_envelope_schema_v1() {
    let output = tyrus_cmd()
        .args([
            "--quiet",
            "check",
            "tests/fixtures/tier1/functions.ts",
            "--json",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("envelope must be valid JSON");
    assert_eq!(json["schemaVersion"], 1, "schemaVersion must be 1");
    assert_eq!(json["status"], "ok", "clean file must report ok");
    assert!(json["errors"].is_array(), "errors must be array");
    assert!(json["diagnostics"].is_array(), "diagnostics must be array");
    assert!(
        json["statementCount"].is_number(),
        "statementCount must be number"
    );
}

#[test]
fn test_cli_quiet_suppresses_banner() {
    let output = tyrus_cmd()
        .args(["--quiet", "check", "tests/fixtures/tier1/functions.ts"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The ASCII art banner contains block characters — should NOT appear with --quiet
    assert!(!stdout.contains("████"));
}

fn rs_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(rs_files_under(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
    found
}

/// Hard analyzer errors must fail `build` on a single file: non-zero exit
/// and no `.rs` artifact written.
#[test]
fn test_cli_build_single_file_fails_on_forbidden_ts() {
    let dir = tempdir().expect("tempdir");
    let out = dir.path().join("out.rs");
    let output = tyrus_cmd()
        .args(["--quiet", "build", "tests/fixtures/uat/forbidden.ts", "-o"])
        .arg(&out)
        .output()
        .expect("failed to run");
    assert!(
        !output.status.success(),
        "build of forbidden TS must exit non-zero"
    );
    assert!(!out.exists(), "forbidden TS must not produce an artifact");
}

/// Hard analyzer errors must fail `build` on a directory: non-zero exit
/// and no generated `.rs` anywhere under the output dir.
#[test]
fn test_cli_build_directory_fails_on_forbidden_ts() {
    let dir = tempdir().expect("tempdir");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    let forbidden =
        std::fs::read_to_string(workspace_root().join("tests/fixtures/uat/forbidden.ts"))
            .expect("read forbidden fixture");
    std::fs::write(src_dir.join("forbidden.ts"), forbidden).expect("write fixture copy");
    let out_dir = dir.path().join("out");
    let output = tyrus_cmd()
        .args(["--quiet", "build"])
        .arg(&src_dir)
        .arg("-o")
        .arg(&out_dir)
        .output()
        .expect("failed to run");
    assert!(
        !output.status.success(),
        "directory build of forbidden TS must exit non-zero"
    );
    assert!(
        rs_files_under(&out_dir).is_empty(),
        "forbidden TS must not produce generated .rs files"
    );
}

/// Hard analyzer errors must fail `compile` before any scaffold is written:
/// `<outdir>/src/main.rs` must not exist.
#[test]
fn test_cli_compile_fails_on_forbidden_ts_without_scaffold() {
    let dir = tempdir().expect("tempdir");
    let output = tyrus_cmd()
        .args([
            "--quiet",
            "compile",
            "tests/fixtures/uat/forbidden.ts",
            "-o",
        ])
        .arg(dir.path())
        .output()
        .expect("failed to run");
    assert!(
        !output.status.success(),
        "compile of forbidden TS must exit non-zero"
    );
    assert!(
        !dir.path().join("src/main.rs").exists(),
        "forbidden TS must not be scaffolded into a cargo project"
    );
}

/// Hard analyzer errors must fail `run` before any scaffold is written:
/// `<outdir>/src/main.rs` must not exist.
#[test]
fn test_cli_run_fails_on_forbidden_ts() {
    let dir = tempdir().expect("tempdir");
    let output = tyrus_cmd()
        .args(["--quiet", "run", "tests/fixtures/uat/forbidden.ts", "-o"])
        .arg(dir.path())
        .output()
        .expect("failed to run");
    assert!(
        !output.status.success(),
        "run of forbidden TS must exit non-zero"
    );
    assert!(
        !dir.path().join("src/main.rs").exists(),
        "forbidden TS must not be scaffolded into a cargo project"
    );
}

/// Soft diagnostics (e.g., setTimeout) must NOT fail `build`: exit zero
/// and the artifact is still produced.
#[test]
fn test_cli_build_succeeds_with_soft_diagnostics_only() {
    let dir = tempdir().expect("tempdir");
    let out = dir.path().join("out.rs");
    let output = tyrus_cmd()
        .args([
            "--quiet",
            "build",
            "tests/fixtures/invalid/uses_settimeout.ts",
            "-o",
        ])
        .arg(&out)
        .output()
        .expect("failed to run");
    assert!(
        output.status.success(),
        "soft diagnostics must not fail build"
    );
    assert!(
        out.exists(),
        "build with soft diagnostics must write the artifact"
    );
}
