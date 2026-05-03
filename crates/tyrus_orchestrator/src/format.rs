use tyrus_diagnostics::TyrusError;

pub(crate) fn format_code(code: &str) -> Result<String, TyrusError> {
    let syntax_tree =
        syn::parse_file(code).map_err(|e| TyrusError::FormattingError(e.to_string()))?;
    Ok(prettyplease::unparse(&syntax_tree))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression gate for #131 — `prettyplease::unparse` must be a fixed point.
    /// Historically a "formatting skipped" bypass for async code violated this:
    /// downstream `cargo fmt --check` saw spurious diffs and the user mistook
    /// the bypass output for current behavior.
    #[test]
    fn format_code_is_idempotent() {
        let inputs = [
            r#"
fn main() {
    println!("hello");
}
"#,
            r#"
struct Foo {
    x: i32,
    y: String,
}

impl Foo {
    fn new() -> Self {
        Self { x: 0, y: String::new() }
    }
}
"#,
            // Async + Result — the historical bypass class.
            r#"
async fn handler() -> Result<String, Box<dyn std::error::Error>> {
    Ok("ok".to_string())
}
"#,
            // Mutex pattern from generated NestJS services.
            r#"
pub struct Svc { x: std::sync::Arc<std::sync::Mutex<f64>> }
impl Svc {
    pub fn read(&self) -> f64 {
        { let __g = self.x.lock().unwrap_or_else(|e| e.into_inner()); *__g }
    }
}
"#,
        ];
        for src in inputs {
            let once = format_code(src).expect("first format must succeed");
            let twice = format_code(&once).expect("second format must succeed");
            assert_eq!(
                once, twice,
                "format_code is not idempotent on:\n--- input ---\n{src}\n--- once ---\n{once}\n--- twice ---\n{twice}",
            );
        }
    }

    /// Invalid Rust must error, never silently pass through. The historical
    /// "formatting skipped for edition compatibility" comment indicated a
    /// fallback that did exactly that — long single-line declarations leaked
    /// to disk and the user couldn't tell formatting had been bypassed.
    #[test]
    fn format_code_propagates_parse_errors() {
        let invalid = "fn (((( this is :: not valid";
        let result = format_code(invalid);
        assert!(
            matches!(result, Err(TyrusError::FormattingError(_))),
            "Expected FormattingError on invalid Rust, got: {result:?}"
        );
    }

    /// Specific regression for the historical async bypass — async code must
    /// flow through prettyplease cleanly with no edition-compat fallback.
    #[test]
    fn format_code_handles_async_without_bypass() {
        let async_code = r#"
async fn fetch() -> Result<String, AppError> {
    let response = reqwest::get("https://example.com").await?;
    Ok(response.text().await?)
}
"#;
        let formatted = format_code(async_code).expect("async must format");
        assert!(formatted.contains("async fn fetch"));
        assert!(formatted.contains(".await"));
        // No bypass marker should ever appear in formatted output.
        assert!(
            !formatted.contains("formatting skipped"),
            "Bypass marker leaked into output: {formatted}"
        );
    }
}

/// Minimal AppError for non-web async code (no axum dependency).
pub(crate) fn get_app_error_simple() -> &'static str {
    r#"
#[derive(Debug)]
pub struct AppError(String);

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<E> From<E> for AppError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Self(err.to_string())
    }
}
"#
}

/// Full AppError with named HTTP variants and axum IntoResponse (for web/NestJS code).
pub(crate) fn get_app_error_code() -> &'static str {
    r#"
use axum::{response::{IntoResponse, Response}, http::StatusCode};

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, message).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Self::Internal(err.to_string())
    }
}
"#
}
