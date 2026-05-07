use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use tyrus_di::graph::DiGraph;
use tyrus_diagnostics::TyrusError;

use crate::format::{format_code, get_app_error_code};
use crate::scaffold::{generate_cargo_toml, generate_main_rs, generate_mod_rs};

mod analyze;

/// Output of phase 1: every parsed `.ts` file plus the class metadata
/// needed by phases 2-5.
struct ParsedProject {
    programs: Vec<swc_ecma_ast::Program>,
    file_paths: Vec<PathBuf>,
    class_module_map: HashMap<String, String>,
    generic_classes: HashSet<String>,
}

/// Inputs that `generate_main_rs` needs in phase 5.
struct MainRsCtx<'a> {
    init_order: &'a [String],
    class_module_map: &'a HashMap<String, String>,
    controllers: &'a [String],
    graph: &'a DiGraph,
    generic_classes: &'a HashSet<String>,
}

pub(crate) fn build_project_impl(input_dir: &Path, output_dir: &Path) -> Result<(), TyrusError> {
    let parsed = parse_and_collect(input_dir)?;
    let (graph, init_order) = analyze::analyze_di_graph(&parsed)?;
    let controllers = transpile_files(input_dir, output_dir, &parsed)?;
    finalize_module_tree(output_dir)?;
    write_main_rs(
        output_dir,
        &MainRsCtx {
            init_order: &init_order,
            class_module_map: &parsed.class_module_map,
            controllers: &controllers,
            graph: &graph,
            generic_classes: &parsed.generic_classes,
        },
    )?;
    generate_cargo_toml(output_dir)?;
    Ok(())
}

// ------- phase 1: walk, parse, collect class metadata -------

fn parse_and_collect(input_dir: &Path) -> Result<ParsedProject, TyrusError> {
    let mut acc = ParsedProject {
        programs: Vec::new(),
        file_paths: Vec::new(),
        class_module_map: HashMap::new(),
        generic_classes: HashSet::new(),
    };

    for entry in WalkDir::new(input_dir) {
        let entry = entry.map_err(|e| TyrusError::IoError(e.into()))?;
        let path = entry.path();
        if !is_typescript_file(path) {
            continue;
        }
        ingest_typescript_file(input_dir, path, &mut acc)?;
    }
    Ok(acc)
}

fn is_typescript_file(path: &Path) -> bool {
    path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ts")
}

fn ingest_typescript_file(
    input_dir: &Path,
    path: &Path,
    acc: &mut ParsedProject,
) -> Result<(), TyrusError> {
    let program = tyrus_parser::parse(path)?;
    let module_path = compute_module_path(input_dir, path)?;
    record_classes(&program, &module_path, acc);
    acc.programs.push(program);
    acc.file_paths.push(path.to_path_buf());
    Ok(())
}

fn compute_module_path(input_dir: &Path, path: &Path) -> Result<String, TyrusError> {
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

    let mut module_parts = vec!["tyrus_app".to_string()];
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
    if sanitized_stem != "index" {
        module_parts.push(sanitized_stem);
    }
    Ok(module_parts.join("::"))
}

fn record_classes(program: &swc_ecma_ast::Program, module_path: &str, acc: &mut ParsedProject) {
    match program {
        swc_ecma_ast::Program::Module(m) => {
            for item in &m.body {
                record_class_from_module_item(item, module_path, acc);
            }
        }
        swc_ecma_ast::Program::Script(s) => {
            for stmt in &s.body {
                if let swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Class(class_decl)) = stmt {
                    insert_class(class_decl, module_path, acc, true);
                }
            }
        }
    }
}

fn record_class_from_module_item(
    item: &swc_ecma_ast::ModuleItem,
    module_path: &str,
    acc: &mut ParsedProject,
) {
    match item {
        swc_ecma_ast::ModuleItem::ModuleDecl(swc_ecma_ast::ModuleDecl::ExportDecl(export)) => {
            if let swc_ecma_ast::Decl::Class(class_decl) = &export.decl {
                insert_class(class_decl, module_path, acc, true);
            }
        }
        swc_ecma_ast::ModuleItem::Stmt(swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Class(
            class_decl,
        ))) => {
            insert_class(class_decl, module_path, acc, false);
        }
        _ => {}
    }
}

fn insert_class(
    class_decl: &swc_ecma_ast::ClassDecl,
    module_path: &str,
    acc: &mut ParsedProject,
    overwrite: bool,
) {
    let class_name = class_decl.ident.sym.to_string();
    if overwrite {
        acc.class_module_map
            .insert(class_name.clone(), module_path.to_string());
    } else {
        acc.class_module_map
            .entry(class_name.clone())
            .or_insert_with(|| module_path.to_string());
    }

    if let Some(type_params) = &class_decl.class.type_params {
        if !type_params.params.is_empty() {
            acc.generic_classes.insert(class_name);
        }
    }
}

// ------- phase 2: DI graph analysis → see pipeline/analyze.rs -------

// ------- phase 3: codegen + write .rs files -------

fn transpile_files(
    input_dir: &Path,
    output_dir: &Path,
    parsed: &ParsedProject,
) -> Result<Vec<String>, TyrusError> {
    let mut controllers = Vec::new();
    for (i, program) in parsed.programs.iter().enumerate() {
        let path = &parsed.file_paths[i];
        write_transpiled_file(input_dir, output_dir, path, program, &mut controllers)?;
    }
    Ok(controllers)
}

fn write_transpiled_file(
    input_dir: &Path,
    output_dir: &Path,
    path: &Path,
    program: &swc_ecma_ast::Program,
    controllers: &mut Vec<String>,
) -> Result<(), TyrusError> {
    let relative_path = path.strip_prefix(input_dir).unwrap_or(path);
    let relative_path = relative_path.strip_prefix("src").unwrap_or(relative_path);
    let output_path = output_dir.join("src").join(relative_path);

    let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let sanitized_stem = file_stem.replace(['.', '-'], "_");
    let is_index = path.file_stem().and_then(|s| s.to_str()) == Some("index");

    let generated = tyrus_codegen::generate(program, is_index);
    let formatted_code = format_code(&generated.code)?;
    let output_file = output_path.with_file_name(format!("{sanitized_stem}.rs"));

    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent).map_err(TyrusError::IoError)?;
    }
    fs::write(output_file, formatted_code).map_err(TyrusError::IoError)?;

    for controller in generated.controllers {
        controllers.push(controller.struct_name);
    }
    Ok(())
}

// ------- phase 4: walk dirs, build mod.rs tree, finalize lib.rs + error.rs -------

fn finalize_module_tree(output_dir: &Path) -> Result<(), TyrusError> {
    for entry in WalkDir::new(output_dir) {
        let entry = entry.map_err(|e| TyrusError::IoError(e.into()))?;
        let path = entry.path();
        if path.is_dir() {
            generate_mod_rs(path)?;
        }
    }

    rename_src_mod_to_lib(output_dir)?;
    write_error_rs(output_dir)?;
    patch_lib_rs_preamble(output_dir)
}

fn rename_src_mod_to_lib(output_dir: &Path) -> Result<(), TyrusError> {
    let src_mod = output_dir.join("src").join("mod.rs");
    let src_lib = output_dir.join("src").join("lib.rs");
    if src_mod.exists() {
        fs::rename(src_mod, src_lib).map_err(TyrusError::IoError)?;
    }
    Ok(())
}

fn write_error_rs(output_dir: &Path) -> Result<(), TyrusError> {
    let error_rs = output_dir.join("src").join("error.rs");
    fs::write(error_rs, get_app_error_code()).map_err(TyrusError::IoError)
}

fn patch_lib_rs_preamble(output_dir: &Path) -> Result<(), TyrusError> {
    let src_lib = output_dir.join("src").join("lib.rs");
    let mut lib_content = fs::read_to_string(&src_lib).map_err(TyrusError::IoError)?;
    lib_content = format!("#![allow(unused)]\n\n{lib_content}");
    if !lib_content.contains("pub mod error;") {
        lib_content.push_str("\npub mod error;\n");
    }
    if !lib_content.contains("pub use error::AppError;") {
        lib_content.push_str("pub use error::AppError;\n");
    }
    fs::write(&src_lib, lib_content).map_err(TyrusError::IoError)
}

// ------- phase 5: main.rs -------

fn write_main_rs(output_dir: &Path, ctx: &MainRsCtx<'_>) -> Result<(), TyrusError> {
    let main_content = generate_main_rs(
        ctx.init_order,
        ctx.class_module_map,
        ctx.controllers,
        ctx.graph,
        ctx.generic_classes,
    )?;

    let src_dir = output_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir).map_err(TyrusError::IoError)?;
    }
    let main_rs = src_dir.join("main.rs");
    fs::write(main_rs, main_content).map_err(TyrusError::IoError)
}
