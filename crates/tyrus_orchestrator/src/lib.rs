use std::path::Path;

use tyrus_common::fs::FilePath;
use tyrus_diagnostics::TyrusError;

mod format;
mod pipeline;
mod scaffold;

pub fn check(path: &FilePath) -> Result<(), TyrusError> {
    let program = tyrus_parser::parse(path.as_ref())?;

    // Read source code for error reporting
    let source_code = std::fs::read_to_string(path.as_ref()).map_err(TyrusError::IoError)?;
    let file_name = path.as_ref().to_string_lossy().to_string();

    let analysis_result = tyrus_analyzer::Analyzer::analyze(&program, source_code, file_name);
    let errors = analysis_result.errors;

    if !errors.is_empty() {
        for error in errors {
            println!("{:?}", miette::Report::new(error));
        }
        return Ok(()); // Or return Err if we want to stop execution
    }

    let count = match program {
        swc_ecma_ast::Program::Module(m) => m.body.len(),
        swc_ecma_ast::Program::Script(s) => s.body.len(),
    };
    println!("✅ AST parsed successfully with {} statements", count);
    Ok(())
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
