#![forbid(unsafe_code)]

use std::path::Path;

use tyrus_common::fs::FilePath;
use tyrus_diagnostics::TyrusError;

mod format;
mod gate;
mod pipeline;
mod scaffold;

#[derive(Debug)]
pub struct CheckResult {
    pub errors: Vec<TyrusError>,
    pub diagnostics: Vec<tyrus_analyzer::severity::Diagnostic>,
    pub statement_count: usize,
}

fn parse_and_analyze(
    path: &FilePath,
) -> Result<(swc_ecma_ast::Program, tyrus_analyzer::AnalysisResult), TyrusError> {
    let program = tyrus_parser::parse(path.as_ref())?;
    let source_code = std::fs::read_to_string(path.as_ref()).map_err(TyrusError::IoError)?;
    let file_name = path.as_ref().to_string_lossy().to_string();
    let analysis = tyrus_analyzer::Analyzer::analyze(&program, source_code, file_name);
    Ok((program, analysis))
}

pub fn check(path: &FilePath) -> Result<CheckResult, TyrusError> {
    let (program, analysis) = parse_and_analyze(path)?;

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
    let (program, analysis) = parse_and_analyze(path)?;
    let lint_error_count = gate::render_findings(analysis.errors, &analysis.diagnostics);
    gate::refuse_on_lint_errors(lint_error_count)?;

    // Default to false for single file build
    let generated_code = tyrus_codegen::generate(&program, false);
    let mut code = generated_code.code;

    // Conditionally inject AppError boilerplate:
    // Only needed when async functions generate `Result<T, crate::AppError>` return types
    let needs_app_error = code.contains("crate::AppError") || code.contains("crate :: AppError");
    if needs_app_error {
        code = code.replace("crate::AppError", "AppError");
        code = code.replace("crate :: AppError", "AppError");
        // Use full axum AppError for web code, simple variant for standalone async
        let has_routing = code.contains("axum::Router") || code.contains("axum::routing");
        if has_routing {
            code.push_str(format::get_app_error_code());
        } else {
            code.push_str(format::get_app_error_simple());
        }
    }

    format::format_code(&code)
}

pub fn build_project(input_dir: &Path, output_dir: &Path) -> Result<(), TyrusError> {
    pipeline::build_project_impl(input_dir, output_dir)
}

/// Build a single file into a standalone Cargo project (no NestJS/axum scaffolding).
pub fn build_simple_project(path: &FilePath, output_dir: &Path) -> Result<(), TyrusError> {
    let code = build(path)?;

    let src_dir = output_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(TyrusError::IoError)?;

    let main_rs = src_dir.join("main.rs");
    std::fs::write(&main_rs, &code).map_err(TyrusError::IoError)?;

    let cargo_toml = output_dir.join("Cargo.toml");
    let deps = detect_dependencies(&code);
    let cargo_content = format!(
        "[package]\nname = \"tyrus_app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
         [workspace]\n\n[[bin]]\nname = \"tyrus_app\"\npath = \"src/main.rs\"\n\n\
         [dependencies]\n{deps}"
    );
    std::fs::write(cargo_toml, cargo_content).map_err(TyrusError::IoError)?;

    Ok(())
}

/// Detect which crate dependencies the generated code needs.
fn detect_dependencies(code: &str) -> String {
    let mut deps = String::new();
    let has_serde_json = code.contains("serde_json");
    if has_serde_json {
        deps.push_str("serde_json = { version = \"1\" }\n");
    }
    // serde is needed for derive macros (Serialize/Deserialize) even without serde_json
    if code.contains("serde::") || code.contains("serde ") || has_serde_json {
        deps.push_str("serde = { version = \"1\", features = [\"derive\"] }\n");
    }
    if code.contains("reqwest::") {
        deps.push_str("reqwest = { version = \"0.12\", features = [\"json\"] }\n");
    }
    if code.contains("tokio::") {
        deps.push_str("tokio = { version = \"1\", features = [\"full\"] }\n");
    }
    // Only add axum when routing constructs are present (not just AppError template)
    if code.contains("axum::") {
        deps.push_str("axum = \"0.7\"\n");
    }
    deps
}
