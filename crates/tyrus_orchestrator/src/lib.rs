use std::path::Path;

use tyrus_common::fs::FilePath;
use tyrus_diagnostics::TyrusError;

mod format;
mod pipeline;
mod scaffold;

pub struct CheckResult {
    pub errors: Vec<TyrusError>,
    pub diagnostics: Vec<tyrus_analyzer::severity::Diagnostic>,
    pub statement_count: usize,
}

pub fn check(path: &FilePath) -> Result<CheckResult, TyrusError> {
    let program = tyrus_parser::parse(path.as_ref())?;

    let source_code = std::fs::read_to_string(path.as_ref()).map_err(TyrusError::IoError)?;
    let file_name = path.as_ref().to_string_lossy().to_string();

    let analysis = tyrus_analyzer::Analyzer::analyze(&program, source_code, file_name);

    let count = match &program {
        swc_ecma_ast::Program::Module(m) => m.body.len(),
        swc_ecma_ast::Program::Script(s) => s.body.len(),
    };

    Ok(CheckResult {
        errors: analysis.errors,
        diagnostics: analysis.diagnostics,
        statement_count: count,
    })
}

pub fn build(path: &FilePath) -> Result<String, TyrusError> {
    let program = tyrus_parser::parse(path.as_ref())?;
    // Default to false for single file build
    let generated_code = tyrus_codegen::generate(&program, false);
    let mut code = generated_code.code;

    // Conditionally inject AppError boilerplate:
    // Only needed when async functions generate `Result<T, crate::AppError>` return types
    if code.contains("crate::AppError") {
        // Replace crate:: prefix with local reference since we're inlining the struct
        code = code.replace("crate::AppError", "AppError");
        code.push_str(format::get_app_error_code());
    } else if code.contains("crate :: AppError") {
        code = code.replace("crate :: AppError", "AppError");
        code.push('\n');
        code.push_str(format::get_app_error_code());
    }

    format::format_code(&code)
}

pub fn build_project(input_dir: &Path, output_dir: &Path) -> Result<(), TyrusError> {
    pipeline::build_project_impl(input_dir, output_dir)
}
