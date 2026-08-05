use std::path::{Path, PathBuf};
use std::process::Command;

use miette::Result;
use tyrus_common::fs::FilePath;

use crate::ui::{colors, progress::Pipeline};

pub(crate) fn execute(path: &Path, output: Option<PathBuf>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("./tyrus_output"));
    let mut pipeline = Pipeline::new(vec!["Transpile", "Compile", "Execute"]);

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

    pipeline.start_step("Building generated Rust project");

    let build_output = Command::new("cargo")
        .args(["build"])
        .current_dir(&output_dir)
        .output()
        .map_err(|e| miette::miette!("Failed to run cargo: {e}"))?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        Pipeline::finish_error("Compilation failed");
        eprintln!("{stderr}");
        return Err(miette::miette!("cargo build failed"));
    }

    pipeline.start_step("Running binary");

    let binary = output_dir.join("target/debug/tyrus_app");
    let run_output = Command::new(&binary)
        .output()
        .map_err(|e| miette::miette!("Failed to execute binary: {e}"))?;

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    let stderr_out = String::from_utf8_lossy(&run_output.stderr);
    if !stdout.is_empty() {
        print!("{stdout}");
    }
    if !stderr_out.is_empty() {
        eprint!("{stderr_out}");
    }

    if run_output.status.success() {
        Pipeline::finish_success("Execution complete");
    } else {
        let code = run_output.status.code().unwrap_or(-1);
        Pipeline::finish_error(&format!("Process exited with code {code}"));
    }

    Ok(())
}
