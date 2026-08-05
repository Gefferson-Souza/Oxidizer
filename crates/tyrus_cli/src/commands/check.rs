use std::path::Path;

use miette::Result;
use tyrus_common::fs::FilePath;
use tyrus_diagnostics::TyrusError;

use crate::ui::{colors, progress::Pipeline};

pub(crate) fn execute(path: &Path, json: bool, strict: bool) -> Result<()> {
    let mut pipeline = Pipeline::new(vec!["Parse", "Analyze", "Report"]);
    pipeline.start_step(&colors::file_path(&path.display().to_string()));

    // Per plan D4 — pre-analysis failures (IO, parse) must still produce
    // a stable JSON envelope when --json is set, so tooling consumers
    // can parse stdout regardless of which stage failed.
    let result = match tyrus_orchestrator::check(&FilePath::from(path.to_path_buf())) {
        Ok(r) => r,
        Err(e) => {
            if json {
                println!("{}", tyrus_analyzer::report::format_json_failure(&e));
            }
            return Err(miette::miette!("{e}"));
        }
    };

    pipeline.start_step("Running Oxidizable checks");

    // Per plan D3 — distinguish hard errors from soft diagnostics.
    let total_fatal = count_fatal(result.errors.len(), result.diagnostics.len(), strict);

    pipeline.start_step("Generating report");

    if json {
        // Per plan D4 — single envelope on stdout. Suppress per-error stderr.
        println!(
            "{}",
            tyrus_analyzer::report::format_json_full(
                &result.errors,
                &result.diagnostics,
                result.statement_count,
            )
        );
    } else {
        for error in result.errors {
            eprintln!("{:?}", miette::Report::new(error));
        }
        if !result.diagnostics.is_empty() {
            eprintln!(
                "{}",
                tyrus_analyzer::report::format_pretty(&result.diagnostics)
            );
        }
    }

    if total_fatal == 0 {
        Pipeline::finish_success(&format!(
            "File is Oxidizable — {} statements parsed",
            result.statement_count,
        ));
        Ok(())
    } else {
        Pipeline::finish_error(&format!("{total_fatal} issue(s) found"));
        // Per plan D5 — exit code primary signal even in --json mode.
        // Aggregate message only; per-error details already printed above.
        Err(TyrusError::Validation(format!("{total_fatal} oxidizable check(s) failed")).into())
    }
}

/// Pure outcome calculation per plan D3. Returns the number of fatal
/// issues for a given `(error_count, diagnostic_count, strict)` tuple.
/// Hard analyzer errors are always fatal; soft diagnostics only become
/// fatal when the user opts into `--strict`.
pub(crate) fn count_fatal(error_count: usize, diagnostic_count: usize, strict: bool) -> usize {
    error_count + if strict { diagnostic_count } else { 0 }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::count_fatal;

    #[test]
    fn d3_zero_when_clean() {
        assert_eq!(count_fatal(0, 0, false), 0);
        assert_eq!(count_fatal(0, 0, true), 0);
    }

    #[test]
    fn d3_hard_errors_always_fatal() {
        assert_eq!(count_fatal(3, 0, false), 3);
        assert_eq!(count_fatal(3, 0, true), 3);
    }

    #[test]
    fn d3_diagnostics_advisory_by_default() {
        assert_eq!(
            count_fatal(0, 5, false),
            0,
            "diagnostics alone don't fail without --strict"
        );
    }

    #[test]
    fn d3_strict_promotes_diagnostics() {
        assert_eq!(count_fatal(0, 5, true), 5);
    }

    #[test]
    fn d3_strict_combines_errors_and_diagnostics() {
        assert_eq!(count_fatal(2, 3, true), 5);
        assert_eq!(count_fatal(2, 3, false), 2);
    }
}
