use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
}

fn tyrus_cmd() -> Command {
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
    tyrus_cmd()
        .args([
            "--quiet",
            "check",
            "tests/fixtures/tier1/functions.ts",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains('['));
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
