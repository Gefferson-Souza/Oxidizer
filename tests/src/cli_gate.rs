//! Analyzer gate contract for the emitting commands (`build`, `compile`,
//! `run`): forbidden TypeScript is refused before any artifact is written,
//! soft diagnostics stay advisory (#188).

use std::path::{Path, PathBuf};
use std::process::Output;

use tempfile::{tempdir, TempDir};

use crate::cli::{tyrus_cmd, workspace_root};

const FORBIDDEN_FIXTURE: &str = "tests/fixtures/uat/forbidden.ts";
const SOFT_ONLY_FIXTURE: &str = "tests/fixtures/invalid/uses_settimeout.ts";
/// Stable code of the aggregated refusal — proves the failure came from the
/// analyzer gate rather than from an IO or parse error.
const GATE_CODE: &str = "tyrus::validation_error";
/// Stable code of the soft diagnostic raised by the soft-only fixture.
const SOFT_DIAGNOSTIC_CODE: &str = "tyrus::unsupported::timer";
const CLEAN_MODULE: &str =
    "export function greet(name: string): string { return \"hi \" + name; }\n";

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

fn fixture_source(relative: &str) -> String {
    std::fs::read_to_string(workspace_root().join(relative)).expect("read fixture")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Creates `<tempdir>/src` with the given `(file name, source)` pairs and
/// returns the tempdir plus the `src` and `out` paths.
fn project_with(files: &[(&str, &str)]) -> (TempDir, PathBuf, PathBuf) {
    let dir = tempdir().expect("tempdir");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    for (name, source) in files {
        std::fs::write(src_dir.join(name), source).expect("write fixture copy");
    }
    let out_dir = dir.path().join("out");
    (dir, src_dir, out_dir)
}

fn build_directory(src_dir: &Path, out_dir: &Path) -> Output {
    tyrus_cmd()
        .args(["--quiet", "build"])
        .arg(src_dir)
        .arg("-o")
        .arg(out_dir)
        .output()
        .expect("failed to run")
}

fn assert_refused_by_gate(output: &Output, what: &str) {
    assert!(!output.status.success(), "{what} must exit non-zero");
    assert!(
        stderr_of(output).contains(GATE_CODE),
        "{what} must be refused by the analyzer gate ({GATE_CODE}), stderr:\n{}",
        stderr_of(output)
    );
}

/// Hard analyzer errors fail a single-file `build`: no `.rs` artifact.
#[test]
fn test_cli_build_single_file_fails_on_forbidden_ts() {
    let dir = tempdir().expect("tempdir");
    let out = dir.path().join("out.rs");
    let output = tyrus_cmd()
        .args(["--quiet", "build", FORBIDDEN_FIXTURE, "-o"])
        .arg(&out)
        .output()
        .expect("failed to run");
    assert_refused_by_gate(&output, "build of forbidden TS");
    assert!(!out.exists(), "forbidden TS must not produce an artifact");
}

/// Without `-o` the generated code goes to stdout, so a refusal must leave
/// stdout empty — findings belong to stderr.
#[test]
fn test_cli_build_without_output_flag_keeps_stdout_empty_on_refusal() {
    let output = tyrus_cmd()
        .args(["--quiet", "build", FORBIDDEN_FIXTURE])
        .output()
        .expect("failed to run");
    assert_refused_by_gate(&output, "build of forbidden TS to stdout");
    assert!(
        output.stdout.is_empty(),
        "refused build must not emit partial code on stdout"
    );
}

/// Hard analyzer errors fail a directory `build` before the output
/// directory even exists.
#[test]
fn test_cli_build_directory_fails_on_forbidden_ts() {
    let forbidden = fixture_source(FORBIDDEN_FIXTURE);
    let (_dir, src_dir, out_dir) = project_with(&[("forbidden.ts", &forbidden)]);
    let output = build_directory(&src_dir, &out_dir);
    assert_refused_by_gate(&output, "directory build of forbidden TS");
    assert!(
        rs_files_under(&out_dir).is_empty() && !out_dir.exists(),
        "forbidden TS must not produce an output directory"
    );
}

/// Lint errors accumulate across files: a clean file walked AFTER a
/// forbidden one must not reset the verdict (the walk is name-sorted).
#[test]
fn test_cli_build_directory_fails_when_only_first_file_is_forbidden() {
    let forbidden = fixture_source(FORBIDDEN_FIXTURE);
    let (_dir, src_dir, out_dir) =
        project_with(&[("a_forbidden.ts", &forbidden), ("z_clean.ts", CLEAN_MODULE)]);
    let output = build_directory(&src_dir, &out_dir);
    assert_refused_by_gate(&output, "directory build with one forbidden file");
    assert!(
        !out_dir.exists(),
        "a project with any forbidden file must not be transpiled"
    );
}

/// Soft diagnostics stay advisory in directory mode: the project is built
/// and the diagnostic is still reported.
#[test]
fn test_cli_build_directory_keeps_soft_diagnostics_advisory() {
    let soft = fixture_source(SOFT_ONLY_FIXTURE);
    let (_dir, src_dir, out_dir) = project_with(&[("timer.ts", &soft)]);
    let output = build_directory(&src_dir, &out_dir);
    assert!(
        output.status.success(),
        "soft diagnostics must not fail a directory build, stderr:\n{}",
        stderr_of(&output)
    );
    assert!(
        !rs_files_under(&out_dir).is_empty(),
        "advisory-only project must be transpiled"
    );
    assert!(
        stderr_of(&output).contains(SOFT_DIAGNOSTIC_CODE),
        "soft diagnostic must still be reported on stderr"
    );
}

/// Hard analyzer errors fail `compile` before any scaffold is written.
#[test]
fn test_cli_compile_fails_on_forbidden_ts_without_scaffold() {
    let dir = tempdir().expect("tempdir");
    let output = tyrus_cmd()
        .args(["--quiet", "compile", FORBIDDEN_FIXTURE, "-o"])
        .arg(dir.path())
        .output()
        .expect("failed to run");
    assert_refused_by_gate(&output, "compile of forbidden TS");
    assert_no_scaffold(dir.path());
}

/// Hard analyzer errors fail `run` before any scaffold is written.
#[test]
fn test_cli_run_fails_on_forbidden_ts() {
    let dir = tempdir().expect("tempdir");
    let output = tyrus_cmd()
        .args(["--quiet", "run", FORBIDDEN_FIXTURE, "-o"])
        .arg(dir.path())
        .output()
        .expect("failed to run");
    assert_refused_by_gate(&output, "run of forbidden TS");
    assert_no_scaffold(dir.path());
}

fn assert_no_scaffold(project_dir: &Path) {
    assert!(
        !project_dir.join("src/main.rs").exists() && !project_dir.join("Cargo.toml").exists(),
        "forbidden TS must not be scaffolded into a cargo project"
    );
}

/// Soft diagnostics (e.g. `setTimeout`) do not fail a single-file `build`:
/// the artifact is produced and the diagnostic is reported.
#[test]
fn test_cli_build_succeeds_with_soft_diagnostics_only() {
    let dir = tempdir().expect("tempdir");
    let out = dir.path().join("out.rs");
    let output = tyrus_cmd()
        .args(["--quiet", "build", SOFT_ONLY_FIXTURE, "-o"])
        .arg(&out)
        .output()
        .expect("failed to run");
    assert!(
        output.status.success(),
        "soft diagnostics must not fail build, stderr:\n{}",
        stderr_of(&output)
    );
    assert!(
        out.exists(),
        "advisory-only input must still produce the artifact"
    );
    assert!(
        stderr_of(&output).contains(SOFT_DIAGNOSTIC_CODE),
        "soft diagnostic must still be reported on stderr"
    );
}
