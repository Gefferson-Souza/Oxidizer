use tyrus_analyzer::severity::Diagnostic;
use tyrus_diagnostics::TyrusError;

/// Prints analyzer findings to stderr (hard errors via miette, soft
/// diagnostics via the shared pretty report) and returns the hard-error
/// count. stderr is mandatory: `build` without `-o` emits generated code
/// on stdout.
#[expect(
    clippy::print_stderr,
    reason = "user-facing analyzer findings; orchestrator has no diagnostics sink yet"
)]
pub(crate) fn render_findings(errors: Vec<TyrusError>, diagnostics: &[Diagnostic]) -> usize {
    let error_count = errors.len();
    for error in errors {
        eprintln!("{:?}", miette::Report::new(error));
    }
    if !diagnostics.is_empty() {
        eprintln!("{}", tyrus_analyzer::report::format_pretty(diagnostics));
    }
    error_count
}

/// Refuses to continue past the analyzer when hard lint errors exist.
/// Soft diagnostics stay advisory — parity with `tyrus check`'s
/// non-strict default.
pub(crate) fn refuse_on_lint_errors(error_count: usize) -> Result<(), TyrusError> {
    if error_count > 0 {
        return Err(TyrusError::Validation(format!(
            "{error_count} oxidizable lint error(s) — run `tyrus check` for details"
        )));
    }
    Ok(())
}
