use console::style;
use miette::Diagnostic as MietteDiagnostic;
use serde_json::{json, Value};
use tyrus_diagnostics::TyrusError;

use crate::severity::{Diagnostic, Severity};

/// Stable schema version of the `--json` envelope.
///
/// Bump policy:
/// * **Non-breaking** (no bump): adding a new field that consumers can ignore.
/// * **Breaking** (bump): renaming/removing a field, changing a field's type,
///   changing the meaning of `status`, or altering the shape of `errors[]` /
///   `diagnostics[]` entries. Document each bump in CHANGELOG.md.
pub const JSON_SCHEMA_VERSION: u32 = 1;

/// Format diagnostics for terminal display
pub fn format_pretty(diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return format!("  {} No issues found\n", style("✓").green().bold());
    }

    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    let infos = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Info)
        .count();

    let mut out = String::new();

    for d in diagnostics {
        let icon = match d.severity {
            Severity::Error => style("✗").red().bold().to_string(),
            Severity::Warning => style("⚠").yellow().bold().to_string(),
            Severity::Info => style("ℹ").blue().bold().to_string(),
        };

        out.push_str(&format!(
            "  {} {} [{}]\n",
            icon,
            d.message,
            style(&d.code).dim()
        ));

        if let Some(span) = &d.span {
            out.push_str(&format!(
                "    at {}:{}..{}\n",
                style(&span.file).underlined(),
                span.start,
                span.end,
            ));
        }

        if let Some(suggestion) = &d.suggestion {
            out.push_str(&format!("    {} {}\n", style("->").cyan(), suggestion));
        }
    }

    out.push_str(&format!(
        "\n  {} error(s), {} warning(s), {} info(s)\n",
        errors, warnings, infos,
    ));

    out
}

/// Convert a `TyrusError` into a stable JSON object. `TyrusError` is not
/// `Serialize`, so we extract the diagnostic `code()` and the `Display` form.
/// Internal helper for the `format_json_*` family.
pub(crate) fn error_to_json_value(err: &TyrusError) -> Value {
    let code = err
        .code()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "tyrus::unknown".to_string());
    json!({
        "code": code,
        "message": format!("{err}"),
    })
}

/// Render an envelope. The fallback path is unreachable in practice
/// (`json!` on integer/string/bool values cannot fail), but R6 forbids
/// `unwrap`/`expect` in production code, so we return a constant-built
/// string referencing `JSON_SCHEMA_VERSION` to avoid drift.
fn render_envelope(envelope: &Value) -> String {
    serde_json::to_string_pretty(envelope).unwrap_or_else(|_| {
        format!(
            "{{\"schemaVersion\":{JSON_SCHEMA_VERSION},\"status\":\"error\",\"errors\":[],\"diagnostics\":[],\"statementCount\":0}}"
        )
    })
}

/// Build the full `--json` envelope used by `tyrus check --json`.
/// Schema is versioned (`schemaVersion`) so external tooling can detect
/// breaking changes. `status` is `"ok"` iff `errors` is empty (per plan D5,
/// `status` reflects hard analyzer errors only; exit code remains the
/// primary signal even under `--strict`).
pub fn format_json_full(
    errors: &[TyrusError],
    diagnostics: &[Diagnostic],
    statement_count: usize,
) -> String {
    let status = if errors.is_empty() { "ok" } else { "error" };
    let envelope = json!({
        "schemaVersion": JSON_SCHEMA_VERSION,
        "status": status,
        "errors": errors.iter().map(error_to_json_value).collect::<Vec<_>>(),
        "diagnostics": diagnostics,
        "statementCount": statement_count,
    });
    render_envelope(&envelope)
}

/// Build a `--json` envelope describing a pre-analysis failure (IO error,
/// parse error). Schema-stable: tooling consumers can rely on the same
/// envelope shape regardless of which stage failed.
pub fn format_json_failure(error: &TyrusError) -> String {
    let empty_diagnostics: Vec<Value> = Vec::new();
    let envelope = json!({
        "schemaVersion": JSON_SCHEMA_VERSION,
        "status": "error",
        "errors": [error_to_json_value(error)],
        "diagnostics": empty_diagnostics,
        "statementCount": 0,
    });
    render_envelope(&envelope)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::severity::Diagnostic;

    fn parse(s: &str) -> serde_json::Value {
        serde_json::from_str(s).expect("envelope must be valid JSON")
    }

    #[test]
    fn envelope_clean_file_reports_status_ok() {
        let json = parse(&format_json_full(&[], &[], 5));
        assert_eq!(json["schemaVersion"], JSON_SCHEMA_VERSION);
        assert_eq!(json["status"], "ok");
        assert_eq!(json["statementCount"], 5);
        assert_eq!(json["errors"].as_array().map(Vec::len), Some(0));
        assert_eq!(json["diagnostics"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn envelope_with_errors_reports_status_error() {
        use miette::NamedSource;
        let var_err = TyrusError::UseOfVar {
            src: NamedSource::new("test.ts", "var x;".to_string()),
            span: (0, 1).into(),
        };
        let json = parse(&format_json_full(&[var_err], &[], 1));
        assert_eq!(json["status"], "error");
        let errors = json["errors"].as_array().expect("errors array");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["code"], "tyrus::lint::no_var");
        assert!(errors[0]["message"].as_str().unwrap_or("").contains("var"));
    }

    #[test]
    fn envelope_with_diagnostics_only_keeps_status_ok() {
        let diag = Diagnostic::error("tyrus::test", "synthetic warning");
        let json = parse(&format_json_full(&[], &[diag], 0));
        // Per D5: status reflects hard errors only; diagnostics do not flip it.
        assert_eq!(json["status"], "ok");
        assert_eq!(json["diagnostics"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn failure_envelope_carries_single_error_entry() {
        let err = TyrusError::Validation("file not found".into());
        let json = parse(&format_json_failure(&err));
        assert_eq!(json["schemaVersion"], JSON_SCHEMA_VERSION);
        assert_eq!(json["status"], "error");
        assert_eq!(json["statementCount"], 0);
        let errors = json["errors"].as_array().expect("errors array");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["code"], "tyrus::validation_error");
    }

    #[test]
    fn error_to_json_value_extracts_code_and_message() {
        let err = TyrusError::Validation("msg".into());
        let v = error_to_json_value(&err);
        assert_eq!(v["code"], "tyrus::validation_error");
        assert!(v["message"].as_str().unwrap_or("").contains("msg"));
    }

    #[test]
    fn pretty_handles_empty_diagnostics() {
        let out = format_pretty(&[]);
        assert!(out.contains("No issues found"));
    }

    #[test]
    fn pretty_renders_severity_counts() {
        let diags = vec![
            Diagnostic::error("tyrus::a", "err"),
            Diagnostic::warning("tyrus::b", "warn"),
        ];
        let out = format_pretty(&diags);
        assert!(out.contains("error(s)"));
        assert!(out.contains("warning(s)"));
    }
}
