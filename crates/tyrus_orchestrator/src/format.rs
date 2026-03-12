use tyrus_diagnostics::TyrusError;

pub(crate) fn format_code(code: &str) -> Result<String, TyrusError> {
    // Skip formatting for code containing async (edition compatibility)
    if code.contains("async fn") {
        return Ok(format!(
            "// Note: async/await code - formatting skipped for edition compatibility\n{}",
            code
        ));
    }

    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(TyrusError::IoError)?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(code.as_bytes())
            .map_err(TyrusError::IoError)?;
    }

    let output = child.wait_with_output().map_err(TyrusError::IoError)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(TyrusError::FormattingError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub(crate) fn get_app_error_code() -> &'static str {
    r#"
use axum::{response::{IntoResponse, Response}, http::StatusCode};

#[derive(Debug)]
pub struct AppError(Box<dyn std::error::Error + Send + Sync>);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            self.0.to_string(),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Self(Box::new(err))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
"#
}
