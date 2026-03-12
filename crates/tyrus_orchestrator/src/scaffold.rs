use std::fs;
use std::path::Path;

use tyrus_diagnostics::TyrusError;

pub(crate) fn generate_main_rs(
    init_order: &[String],
    class_module_map: &std::collections::HashMap<String, String>,
    controllers: &[String],
    graph: &tyrus_di::graph::DiGraph,
    generic_classes: &std::collections::HashSet<String>,
) -> Result<String, TyrusError> {
    let mut main_content = String::new();
    main_content.push_str("#![allow(unused)]\n\n");
    main_content.push_str("use axum::Router;\n");
    main_content.push_str("use tokio::net::TcpListener;\n");
    main_content.push_str("use std::sync::Arc;\n");
    main_content.push_str("use axum::Extension;\n\n");

    main_content.push_str("#[tokio::main]\n");
    main_content.push_str("async fn main() -> Result<(), Box<dyn std::error::Error>> {\n");

    // Instantiate components in order
    let mut instantiated_vars = std::collections::HashMap::new();

    for class_name in init_order {
        if generic_classes.contains(class_name) {
            continue;
        }
        if let Some(module_path) = class_module_map.get(class_name) {
            let var_name = tyrus_common::util::to_snake_case(class_name);

            // Get dependencies
            let deps = graph
                .get_provider_dependencies(class_name)
                .unwrap_or_default();
            let mut args = Vec::new();
            for dep in deps {
                let dep_var = tyrus_common::util::to_snake_case(&dep);
                args.push(format!("{}.clone()", dep_var));
            }

            // Check if it has new_di
            // For now assume yes if it has dependencies, or just call new_di
            main_content.push_str(&format!(
                "    let {} = Arc::new({}::{}::new_di({}));\n",
                var_name,
                module_path,
                class_name,
                args.join(", ")
            ));

            instantiated_vars.insert(class_name.clone(), var_name);
        }
    }

    main_content.push_str("\n    // Build router\n");
    main_content.push_str("    let app = axum::Router::new()");

    // Register controllers
    for controller in controllers {
        if let Some(module_path) = class_module_map.get(controller) {
            main_content.push_str(&format!(
                "\n        .merge({}::{}::router())",
                module_path, controller
            ));
        }
    }

    // Add extensions
    for var_name in instantiated_vars.values() {
        main_content.push_str(&format!(
            "\n        .layer(Extension({}.clone()))",
            var_name
        ));
    }

    main_content.push_str(";\n\n");
    main_content.push_str("    let listener = TcpListener::bind(\"0.0.0.0:3000\").await?;\n");
    main_content.push_str("    println!(\"Server running on http://0.0.0.0:3000\");\n");
    main_content.push_str("    axum::serve(listener, app.into_make_service())\n");
    main_content.push_str("        .await?;\n");
    main_content.push_str("    Ok(())\n");
    main_content.push_str("}\n");

    Ok(main_content)
}

pub(crate) fn generate_cargo_toml(output_dir: &Path) -> Result<(), TyrusError> {
    let cargo_toml_content = r#"[package]
name = "tyrus_app"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json"] }
tower = { version = "0.5" }
tower-http = { version = "0.5", features = ["trace"] }
rand = "0.8"

[[bin]]
name = "server"
path = "src/main.rs"

[lib]
name = "tyrus_app"
path = "src/lib.rs"
"#;

    let cargo_toml_path = output_dir.join("Cargo.toml");
    fs::write(cargo_toml_path, cargo_toml_content).map_err(TyrusError::IoError)?;
    Ok(())
}

pub(crate) fn generate_mod_rs(dir: &Path) -> Result<(), TyrusError> {
    let mut mod_content = String::new();
    let mut has_children = false;
    let mut index_content = String::new();

    // Read directory entries
    let entries = fs::read_dir(dir).map_err(TyrusError::IoError)?;

    for entry in entries {
        let entry = entry.map_err(TyrusError::IoError)?;
        let path = entry.path();

        // Skip mod.rs, lib.rs, and main.rs
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "mod.rs" || name == "lib.rs" || name == "main.rs" {
                continue;
            }
        }

        if path.is_dir() {
            // If it's a directory, it should have a mod.rs inside (handled by recursion/iteration),
            // so we expose it as a module.
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let sanitized_name = name.replace(['.', '-'], "_");
                mod_content.push_str(&format!("pub mod {};\n", sanitized_name));
                has_children = true;
            }
        } else if let Some(ext) = path.extension() {
            if ext == "rs" {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem == "index" {
                        // Capture index.rs content to append later
                        index_content = fs::read_to_string(&path).map_err(TyrusError::IoError)?;
                        // We will remove index.rs later or just ignore it in the final build?
                        // Usually we want to remove it so we don't have both mod.rs and index.rs
                        // But let's delete it after reading.
                    } else {
                        // Sanitize module name just in case, though we sanitized filename on write
                        let sanitized_stem = stem.replace(['.', '-'], "_");
                        mod_content.push_str(&format!("pub mod {};\n", sanitized_stem));
                        has_children = true;
                    }
                }
            }
        }
    }

    // If we found index.rs, delete it and append its content
    let index_path = dir.join("index.rs");
    if index_path.exists() {
        fs::remove_file(index_path).map_err(TyrusError::IoError)?;
    }

    // Append index content
    if !index_content.is_empty() {
        mod_content.push('\n');
        mod_content.push_str("// Content from index.ts\n");
        mod_content.push_str(&index_content);
        has_children = true;
    }

    // Only write mod.rs if there's something to put in it (or if it's the root output dir?)
    // Actually, Rust needs mod.rs to expose children. If a dir has children, it needs mod.rs.
    if has_children {
        let mod_path = dir.join("mod.rs");
        fs::write(mod_path, mod_content).map_err(TyrusError::IoError)?;
    }

    Ok(())
}
