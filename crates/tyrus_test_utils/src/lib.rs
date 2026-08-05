#![forbid(unsafe_code)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::string_slice
)]
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tempfile::TempDir;

static RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Shared target directory for compilation caching.
/// Dependencies are compiled once and reused across all tests via `CARGO_TARGET_DIR`.
static SHARED_TARGET_DIR: OnceLock<PathBuf> = OnceLock::new();

fn get_shared_target_dir() -> &'static PathBuf {
    SHARED_TARGET_DIR.get_or_init(|| {
        let target_dir = std::env::temp_dir().join("tyrus_test_target");
        fs::create_dir_all(&target_dir).expect("Failed to create shared target dir");
        target_dir
    })
}

/// Asserts that the provided Rust code compiles successfully as a **library**.
///
/// Each test gets its own temporary project directory (no race conditions),
/// but all tests share a common `CARGO_TARGET_DIR` for dependency caching.
/// Dependencies are compiled once on first use and reused by subsequent tests.
///
/// # Panics
/// Panics if `cargo check` fails, printing the full `rustc` error output.
pub fn assert_rust_compiles(code: &str) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();
    let shared_target = get_shared_target_dir();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");

    let cargo_toml = r#"
[package]
name = "tyrus_app"
version = "0.1.0"
edition = "2021"

[lib]
name = "tyrus_app"
path = "src/lib.rs"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
reqwest = { version = "0.12", features = ["json"] }
tower = { version = "0.5" }
tower-http = { version = "0.5", features = ["trace"] }
rand = "0.8"
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    // Wrap code with common allows to suppress dead_code warnings
    let wrapped_code = format!("#![allow(dead_code, unused_variables, unused_imports)]\n{code}");

    fs::write(src_dir.join("lib.rs"), &wrapped_code).expect("Failed to write lib.rs");

    let output = Command::new("cargo")
        .arg("check")
        .env("CARGO_TARGET_DIR", shared_target)
        .current_dir(project_path)
        .output()
        .expect("Failed to execute cargo check");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "\n╔══════════════════════════════════════════╗\n\
             ║   RUST COMPILATION FAILED                ║\n\
             ╚══════════════════════════════════════════╝\n\n\
             CODE:\n------\n{code}\n------\n\n\
             STDERR:\n{stderr}\n\nSTDOUT:\n{stdout}"
        );
    }
}

/// Sets up a single-binary Cargo project for `code` and runs `cargo build`,
/// returning the path to the produced binary inside the shared target dir.
///
/// `_temp_dir` is dropped on return but the binary lives in the shared target,
/// so the path remains valid for the caller to invoke.
fn prepare_and_build_binary(code: &str) -> PathBuf {
    let run_id = RUN_COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let bin_name = format!("tyrus_run_{pid}_{run_id}");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();
    let shared_target = get_shared_target_dir();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");

    let cargo_toml = format!(
        r#"
[package]
name = "{bin_name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "{bin_name}"
path = "src/main.rs"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#
    );

    fs::write(project_path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    let wrapped_code =
        format!("#![allow(dead_code, unused_variables, unused_imports, unused_mut)]\n{code}");
    fs::write(src_dir.join("main.rs"), &wrapped_code).expect("Failed to write main.rs");

    let build_output = Command::new("cargo")
        .args(["build", "--quiet"])
        .env("CARGO_TARGET_DIR", shared_target.as_path())
        .current_dir(project_path)
        .output()
        .expect("Failed to execute cargo build");

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        panic!("\n=== RUST BUILD FAILED ===\nCODE:\n{code}\n\nSTDERR:\n{stderr}");
    }

    shared_target.join("debug").join(&bin_name)
}

/// Compiles Rust code as a binary and runs it, returning stdout.
/// The code must contain a `fn main()`.
///
/// # Panics
/// Panics if compilation or execution fails. Has no timeout — a deadlocked
/// binary will hang the test runner. Prefer [`compile_and_run_rust_with_timeout`]
/// for code paths that may deadlock.
pub fn compile_and_run_rust(code: &str) -> String {
    let bin_path = prepare_and_build_binary(code);

    let run_output = Command::new(&bin_path)
        .output()
        .expect("Failed to run compiled binary");

    if !run_output.status.success() {
        let stderr = String::from_utf8_lossy(&run_output.stderr);
        panic!("\n=== RUST EXECUTION FAILED ===\nCODE:\n{code}\n\nSTDERR:\n{stderr}");
    }

    String::from_utf8_lossy(&run_output.stdout).to_string()
}

/// Compiles and runs Rust code with a wall-clock timeout on the binary execution.
///
/// On timeout, the child process is killed before this function panics — preventing
/// orphaned processes from accumulating across deadlock-prone tests.
///
/// Build time is **not** included in the timeout; only the binary's execution.
///
/// # Panics
/// - Build failure (same as [`compile_and_run_rust`]).
/// - Non-zero exit status from the binary.
/// - Timeout: the binary did not exit within `timeout` (likely deadlock).
pub fn compile_and_run_rust_with_timeout(code: &str, timeout: Duration) -> String {
    let bin_path = prepare_and_build_binary(code);

    let mut child = Command::new(&bin_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn compiled binary");

    let start = Instant::now();
    let exit_status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "\n=== RUST BINARY TIMEOUT ===\n\
                         Did not exit within {timeout:?} (likely deadlock).\n\
                         CODE:\n{code}"
                    );
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("Failed to poll child process: {e}");
            }
        }
    };

    let output = child
        .wait_with_output()
        .expect("Failed to read child output after exit");

    if !exit_status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("\n=== RUST EXECUTION FAILED ===\nCODE:\n{code}\n\nSTDERR:\n{stderr}");
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Runs TypeScript code using Node.js and returns stdout.
/// Uses `--experimental-strip-types` for native TS support (Node 22+).
///
/// # Panics
/// Panics if Node.js is not installed or execution fails.
pub fn run_node(ts_code: &str) -> String {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ts_file = temp_dir.path().join("test.ts");
    fs::write(&ts_file, ts_code).expect("Failed to write TS file");

    let output = Command::new("node")
        .args([
            "--experimental-strip-types",
            ts_file.to_str().unwrap_or_default(),
        ])
        .output()
        .expect("Node.js not found. Install Node.js 22+ to run equivalence tests.");

    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout).to_string();
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    panic!(
        "\n=== NODE EXECUTION FAILED ===\n\
         Node.js 22+ is required for --experimental-strip-types.\n\
         Check your Node version with: node --version\n\n\
         CODE:\n{ts_code}\n\nSTDERR:\n{stderr}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "RUST BINARY TIMEOUT")]
    fn timeout_kills_deadlocked_binary() {
        compile_and_run_rust_with_timeout(
            r"fn main() { std::thread::park(); }",
            Duration::from_millis(500),
        );
    }

    #[test]
    fn no_timeout_for_fast_program() {
        let out = compile_and_run_rust_with_timeout(
            r#"fn main() { println!("ok"); }"#,
            Duration::from_mins(1),
        );
        assert_eq!(out.trim(), "ok");
    }
}
