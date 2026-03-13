use std::path::Path;

use miette::Result;
use tyrus_common::fs::FilePath;

use crate::ui::{colors, progress::Pipeline};

pub(crate) fn execute(path: &Path, json: bool) -> Result<()> {
    let mut pipeline = Pipeline::new(vec!["Parse", "Analyze", "Report"]);

    pipeline.start_step(&colors::file_path(&path.display().to_string()));

    let result = tyrus_orchestrator::check(&FilePath::from(path.to_path_buf()))
        .map_err(|e| miette::miette!("{e}"))?;

    pipeline.start_step("Running Oxidizable checks");

    // Show TyrusErrors (lint errors)
    let error_count = result.errors.len();
    for error in result.errors {
        eprintln!("{:?}", miette::Report::new(error));
    }

    // Show structured diagnostics (unsupported APIs, warnings)
    pipeline.start_step("Generating report");

    if json {
        println!(
            "{}",
            tyrus_analyzer::report::format_json(&result.diagnostics)
        );
    } else if !result.diagnostics.is_empty() {
        eprintln!(
            "{}",
            tyrus_analyzer::report::format_pretty(&result.diagnostics)
        );
    }

    let total_issues = error_count + result.diagnostics.len();
    if total_issues == 0 {
        pipeline.finish_success(&format!(
            "File is Oxidizable — {} statements parsed",
            result.statement_count,
        ));
    } else {
        pipeline.finish_error(&format!("{total_issues} issue(s) found"));
    }

    Ok(())
}
