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

    pub fn with_span(mut self, start: usize, end: usize, file: &str) -> Self {
        self.span = Some(DiagnosticSpan {
            start,
            end,
            file: file.to_string(),
        });
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }
}
