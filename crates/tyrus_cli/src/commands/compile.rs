use std::path::{Path, PathBuf};
use std::process::Command;

use miette::Result;
use tyrus_common::fs::FilePath;

use crate::ui::{colors, progress::Pipeline};

pub(crate) fn execute(path: &Path, output: Option<PathBuf>, release: bool) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("./tyrus_output"));
    let mut pipeline = Pipeline::new(vec!["Transpile", "Compile"]);

    pipeline.start_step(&format!(
        "{} → {}",
        colors::file_path(&path.display().to_string()),
        colors::file_path(&output_dir.display().to_string()),
    ));

    if path.is_dir() {
        tyrus_orchestrator::build_project(path, &output_dir)?;
    } else {
        tyrus_orchestrator::build_simple_project(&FilePath::from(path.to_path_buf()), &output_dir)?;
    }

    pipeline.start_step("Running cargo build on generated Rust");

    let mut cmd = Command::new("cargo");
    cmd.arg("build").current_dir(&output_dir);
    if release {
        cmd.arg("--release");
    }

    let status = cmd
        .output()
        .map_err(|e| miette::miette!("Failed to run cargo: {e}"))?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        pipeline.finish_error("Compilation failed");
        eprintln!("{stderr}");
        return Err(miette::miette!("cargo build failed"));
    }

    let mode = if release { "release" } else { "debug" };
    pipeline.finish_success(&format!(
        "Compiled → {}/target/{mode}/tyrus_app",
        colors::file_path(&output_dir.display().to_string()),
    ));
    Ok(())
}
