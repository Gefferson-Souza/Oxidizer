#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use tempfile::TempDir;

/// Shared target directory for compilation caching.
/// Dependencies are compiled once and reused across all tests via CARGO_TARGET_DIR.
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
/// but all tests share a common CARGO_TARGET_DIR for dependency caching.
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
    let wrapped_code = format!(
        "#![allow(dead_code, unused_variables, unused_imports)]\n{}",
        code
    );

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
             CODE:\n------\n{}\n------\n\n\
             STDERR:\n{}\n\nSTDOUT:\n{}",
            code, stderr, stdout
        );
    }
}
