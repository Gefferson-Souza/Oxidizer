use std::path::{Path, PathBuf};

use miette::Result;
use tyrus_common::fs::FilePath;

use crate::ui::{colors, progress::Pipeline};

pub(crate) fn execute(path: &Path, output: Option<PathBuf>) -> Result<()> {
    if path.is_dir() {
        execute_project(path, output)
    } else {
        execute_single(path, output)
    }
}

fn execute_project(path: &Path, output: Option<PathBuf>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("./tyrus_output"));
    let mut pipeline = Pipeline::new(vec!["Scan", "Parse", "Analyze", "Transpile", "Scaffold"]);

    pipeline.start_step(&format!(
        "{} → {}",
        colors::file_path(&path.display().to_string()),
        colors::file_path(&output_dir.display().to_string()),
    ));

    pipeline.start_step("Walking TypeScript files");
    pipeline.start_step("Running Oxidizable checks");
    pipeline.start_step("Generating Rust code");
    pipeline.start_step("Creating Cargo project");

    tyrus_orchestrator::build_project(path, &output_dir)?;
    Pipeline::finish_success(&format!(
        "Project built → {}",
        colors::file_path(&output_dir.display().to_string()),
    ));
    Ok(())
}

fn execute_single(path: &Path, output: Option<PathBuf>) -> Result<()> {
    let mut pipeline = Pipeline::new(vec!["Parse", "Transpile", "Format"]);

    pipeline.start_step(&colors::file_path(&path.display().to_string()));
    pipeline.start_step("Generating Rust code");

    let output_code = tyrus_orchestrator::build(&FilePath::from(path.to_path_buf()))?;

    pipeline.start_step("Formatting output");

    if let Some(output_path) = output {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| miette::miette!("Failed to create directory: {e}"))?;
        }
        std::fs::write(&output_path, &output_code)
            .map_err(|e| miette::miette!("Failed to write output file: {e}"))?;
        Pipeline::finish_success(&format!(
            "Built → {}",
            colors::file_path(&output_path.display().to_string()),
        ));
    } else {
        println!("{output_code}");
    }
    Ok(())
}
