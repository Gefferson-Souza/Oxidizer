#[cfg(test)]
mod nestjs_tests {
    use ox_common::fs::FilePath;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_nestjs_controller_generation() {
        let ts_path = PathBuf::from("fixtures/nestjs_controller/src/cats.controller.ts");
        let rust_code =
            ox_orchestrator::build(FilePath::from(ts_path)).expect("Failed to generate Rust code");

        println!("Generated Rust Code:\n{}", rust_code);

        // Verify Axum handlers
        assert!(rust_code.contains("pub async fn find_all"));
        assert!(rust_code.contains("-> String"));
        assert!(rust_code.contains("Route:"));
        assert!(rust_code.contains("GET"));

        assert!(rust_code.contains("pub async fn create"));
        assert!(rust_code.contains("axum :: Json (create_cat_dto)"));
        assert!(rust_code.contains("axum :: Json < CreateCatDto >"));
        assert!(rust_code.contains("-> axum :: Json < CreateCatDto >"));
        assert!(rust_code.contains("POST"));
        assert!(rust_code.contains("return axum :: Json (create_cat_dto)"));
    }

    #[test]
    fn test_cargo_toml_generation() {
        let temp_dir = TempDir::new().unwrap();
        let input_dir = PathBuf::from("fixtures/nestjs_controller/src");
        let output_dir = temp_dir.path().to_path_buf();

        ox_orchestrator::build_project(input_dir, output_dir.clone())
            .expect("Failed to build project");

        let cargo_toml_path = output_dir.join("Cargo.toml");
        assert!(cargo_toml_path.exists());

        let content = std::fs::read_to_string(cargo_toml_path).unwrap();
        assert!(content.contains("[dependencies]"));
        assert!(content.contains("axum = \"0.7\""));
        assert!(content.contains("tokio = { version = \"1.0\", features = [\"full\"] }"));
    }

    #[test]
    fn test_nestjs_project_compilation() {
        // This test actually tries to compile the generated project with cargo
        // It requires internet access to fetch dependencies (axum, tokio, etc.)
        // If this fails due to network, we might need to mock or skip it in CI.

        let temp_dir = TempDir::new().unwrap();
        let input_dir = PathBuf::from("fixtures/nestjs_controller/src");
        let output_dir = temp_dir.path().join("src"); // Put code in src/

        // Create project root
        let project_root = temp_dir.path();
        std::fs::create_dir_all(&output_dir).unwrap();

        // Build project
        ox_orchestrator::build_project(input_dir, output_dir.clone())
            .expect("Failed to build project");

        // Generate Cargo.toml in project root (not in src)
        // Note: build_project generates Cargo.toml in output_dir (src). We need to move it or generate it in root.
        // The current implementation of build_project generates Cargo.toml in the output_dir.
        // If output_dir is `.../src`, then Cargo.toml is in `.../src/Cargo.toml`.
        // Cargo expects Cargo.toml at root.

        let src_cargo = output_dir.join("Cargo.toml");
        let root_cargo = project_root.join("Cargo.toml");
        std::fs::rename(src_cargo, root_cargo).expect("Failed to move Cargo.toml");

        // Rename root mod.rs to lib.rs to make it a library
        let root_mod = output_dir.join("mod.rs");
        let lib_rs = output_dir.join("lib.rs");
        if root_mod.exists() {
            std::fs::rename(root_mod, lib_rs).expect("Failed to rename mod.rs to lib.rs");
        } else {
            // If no mod.rs, maybe just one file?
            // In our fixture, we have cats.controller.ts, so we expect cats_controller.rs and mod.rs
        }

        // Run cargo build (stronger verification than check)
        let status = std::process::Command::new("cargo")
            .arg("build")
            .current_dir(project_root)
            .status()
            .expect("Failed to run cargo build");

        assert!(status.success(), "Generated project failed to compile");
    }
}
