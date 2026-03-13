use console::style;

use crate::severity::{Diagnostic, Severity};

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

/// Format diagnostics as JSON (for tooling integration / future VSCode extension)
pub fn format_json(diagnostics: &[Diagnostic]) -> String {
    serde_json::to_string_pretty(diagnostics).unwrap_or_else(|_| "[]".to_string())
}
