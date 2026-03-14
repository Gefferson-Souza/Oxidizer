use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use tyrus_diagnostics::TyrusError;

use crate::format::{format_code, get_app_error_code};
use crate::scaffold::{generate_cargo_toml, generate_main_rs, generate_mod_rs};

pub(crate) fn build_project_impl(input_dir: &Path, output_dir: &Path) -> Result<(), TyrusError> {
    let mut controllers: Vec<String> = Vec::new(); // Just names of controllers
    let mut class_module_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut generic_classes: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut programs = Vec::new();
    let mut file_paths = Vec::new();

    // 1. Walk, Parse, and Collect Info
    for entry in WalkDir::new(input_dir) {
        let entry = entry.map_err(|e| TyrusError::IoError(e.into()))?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ts") {
            let program = tyrus_parser::parse(path)?;

            // Calculate module path
            let relative_path = path.strip_prefix(input_dir).unwrap_or(path);
            let file_stem = path
                .file_stem()
                .ok_or_else(|| {
                    TyrusError::IoError(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid file path",
                    ))
                })?
                .to_string_lossy();
            let sanitized_stem = file_stem.replace(['.', '-'], "_");

            let mut module_parts = Vec::new();
            module_parts.push("tyrus_app".to_string()); // Use lib crate name

            if let Some(parent) = relative_path.parent() {
                for part in parent.components() {
                    if let std::path::Component::Normal(s) = part {
                        let s_str = s.to_string_lossy();
                        if s_str != "src" {
                            module_parts.push(s_str.replace('-', "_"));
                        }
                    }
                }
            }
            // index.ts → lib.rs (crate root), don't add "index" to module path
            if sanitized_stem != "index" {
                module_parts.push(sanitized_stem);
            }
            let module_path = module_parts.join("::");

            // Extract classes to map them
            if let swc_ecma_ast::Program::Module(m) = &program {
                for item in &m.body {
                    // Exported class declarations
                    if let swc_ecma_ast::ModuleItem::ModuleDecl(
                        swc_ecma_ast::ModuleDecl::ExportDecl(export),
                    ) = item
                    {
                        if let swc_ecma_ast::Decl::Class(class_decl) = &export.decl {
                            let class_name = class_decl.ident.sym.to_string();
                            class_module_map.insert(class_name.clone(), module_path.clone());

                            if let Some(type_params) = &class_decl.class.type_params {
                                if !type_params.params.is_empty() {
                                    generic_classes.insert(class_name);
                                }
                            }
                        }
                    }
                    // Non-exported class declarations (common in NestJS single-file)
                    if let swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Decl(
                        swc_ecma_ast::Decl::Class(class_decl),
                    )) = item
                    {
                        let class_name = class_decl.ident.sym.to_string();
                        class_module_map
                            .entry(class_name.clone())
                            .or_insert_with(|| module_path.clone());

                        if let Some(type_params) = &class_decl.class.type_params {
                            if !type_params.params.is_empty() {
                                generic_classes.insert(class_name);
                            }
                        }
                    }
                }
            } else if let swc_ecma_ast::Program::Script(s) = &program {
                for stmt in &s.body {
                    if let swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Class(class_decl)) = stmt {
                        let class_name = class_decl.ident.sym.to_string();
                        class_module_map.insert(class_name.clone(), module_path.clone());

                        if let Some(type_params) = &class_decl.class.type_params {
                            if !type_params.params.is_empty() {
                                generic_classes.insert(class_name);
                            }
                        }
                    }
                }
            }

            programs.push(program);
            file_paths.push(path.to_path_buf());
        }
    }

    // 2. Analyze (Build Dependency Graph)
    let mut graph = tyrus_di::graph::DiGraph::new();

    // We already parsed programs in the loop above.
    // Ideally we analyze inside the loop, but we need to iterate again or move analysis into step 1.
    // Let's iterate programs again since we have them in memory.
    for (i, program) in programs.iter().enumerate() {
        let path = &file_paths[i];
        let source_code = std::fs::read_to_string(path).map_err(TyrusError::IoError)?;
        let file_name = path.to_string_lossy().to_string();

        let analysis_result = tyrus_analyzer::Analyzer::analyze(program, source_code, file_name);
        for error in analysis_result.errors {
            println!("Warning: {:?}", miette::Report::new(error));
        }
        if !analysis_result.diagnostics.is_empty() {
            eprintln!(
                "{}",
                tyrus_analyzer::report::format_pretty(&analysis_result.diagnostics)
            );
        }

        graph.merge(analysis_result.graph);
    }

    let init_order = graph
        .resolve()
        .map_err(|e| TyrusError::Validation(e.to_string()))?;

    // 3. Transpile
    for (i, program) in programs.iter().enumerate() {
        let path = &file_paths[i];
        let relative_path = path.strip_prefix(input_dir).unwrap_or(path);
        let relative_path = relative_path.strip_prefix("src").unwrap_or(relative_path);
        let output_path = output_dir.join("src").join(relative_path);

        // Calculate module path for this file
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let sanitized_stem = file_stem.replace(['.', '-'], "_");

        // Check if it's index.ts
        let is_index = path.file_stem().and_then(|s| s.to_str()) == Some("index");

        let generated = tyrus_codegen::generate(program, is_index);
        let formatted_code = format_code(&generated.code)?;

        let output_file = output_path.with_file_name(format!("{}.rs", sanitized_stem));

        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent).map_err(TyrusError::IoError)?;
        }

        fs::write(output_file, formatted_code).map_err(TyrusError::IoError)?;

        // Collect controllers
        for controller in generated.controllers {
            controllers.push(controller.struct_name);
        }
    }

    // 4. Generate mod.rs
    for entry in WalkDir::new(output_dir) {
        let entry = entry.map_err(|e| TyrusError::IoError(e.into()))?;
        let path = entry.path();
        if path.is_dir() {
            generate_mod_rs(path)?;
        }
    }

    // Rename src/mod.rs to src/lib.rs for the crate root
    let src_mod = output_dir.join("src").join("mod.rs");
    let src_lib = output_dir.join("src").join("lib.rs");
    if src_mod.exists() {
        fs::rename(src_mod, src_lib.clone()).map_err(TyrusError::IoError)?;
    }

    // Generate error.rs
    let error_rs = output_dir.join("src").join("error.rs");
    let error_content = get_app_error_code();
    fs::write(error_rs, error_content).map_err(TyrusError::IoError)?;

    // Prepend #![allow(unused)] and append mod error + pub use error::AppError to lib.rs
    let mut lib_content = fs::read_to_string(&src_lib).map_err(TyrusError::IoError)?;
    // Add crate-level allow(unused) to suppress NestJS module import warnings
    lib_content = format!("#![allow(unused)]\n\n{}", lib_content);
    // Only add `pub mod error;` if not already present (generate_mod_rs may have added it)
    if !lib_content.contains("pub mod error;") {
        lib_content.push_str("\npub mod error;\n");
    }
    if !lib_content.contains("pub use error::AppError;") {
        lib_content.push_str("pub use error::AppError;\n");
    }
    fs::write(&src_lib, lib_content).map_err(TyrusError::IoError)?;

    // 5. Generate main.rs
    let main_content = generate_main_rs(
        &init_order,
        &class_module_map,
        &controllers,
        &graph,
        &generic_classes,
    )?;

    // Ensure src directory exists
    let src_dir = output_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir).map_err(TyrusError::IoError)?;
    }

    let main_rs = src_dir.join("main.rs");
    fs::write(main_rs, main_content).map_err(TyrusError::IoError)?;

    // 6. Generate Cargo.toml
    generate_cargo_toml(output_dir)?;

    Ok(())
}
