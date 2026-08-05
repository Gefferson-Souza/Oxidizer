/// Analyzer diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Severity {
    /// Must fix — code cannot be transpiled
    Error,
    /// Should fix — code may produce unexpected results
    Warning,
    /// Informational — suggestion for better patterns
    Info,
}

/// A structured diagnostic from the analyzer
#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub span: Option<DiagnosticSpan>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiagnosticSpan {
    pub start: usize,
    pub end: usize,
    pub file: String,
}

impl Diagnostic {
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            severity: Severity::Error,
            span: None,
            suggestion: None,
        }
    }

    pub fn warning(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            severity: Severity::Warning,
            span: None,
            suggestion: None,
        }
    }

    // Why allowed: API surface companion to Self::error / Self::warning.
    // Kept public for future Info-level diagnostics; no callers yet.
    #[allow(dead_code)]
    pub fn info(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            severity: Severity::Info,
            span: None,
            suggestion: None,
        }
    }

    #[must_use]
    pub fn with_span(mut self, start: usize, end: usize, file: &str) -> Self {
        self.span = Some(DiagnosticSpan {
            start,
            end,
            file: file.to_string(),
        });
        self
    }

    #[must_use]
    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn error_constructor_sets_severity() {
        let d = Diagnostic::error("tyrus::test", "boom");
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.code, "tyrus::test");
        assert_eq!(d.message, "boom");
        assert!(d.span.is_none());
        assert!(d.suggestion.is_none());
    }

    #[test]
    fn warning_constructor_sets_severity() {
        let d = Diagnostic::warning("tyrus::test", "soft");
        assert_eq!(d.severity, Severity::Warning);
    }

    #[test]
    fn info_constructor_sets_severity() {
        let d = Diagnostic::info("tyrus::test", "fyi");
        assert_eq!(d.severity, Severity::Info);
    }

    #[test]
    fn with_span_attaches_location() {
        let d = Diagnostic::error("c", "m").with_span(10, 20, "test.ts");
        let span = d.span.expect("span set");
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.file, "test.ts");
    }

    #[test]
    fn with_suggestion_attaches_hint() {
        let d = Diagnostic::warning("c", "m").with_suggestion("try X");
        assert_eq!(d.suggestion.as_deref(), Some("try X"));
    }

    #[test]
    fn builders_chain() {
        let d = Diagnostic::error("c", "m")
            .with_span(0, 5, "f.ts")
            .with_suggestion("hint");
        assert!(d.span.is_some());
        assert!(d.suggestion.is_some());
    }
}
