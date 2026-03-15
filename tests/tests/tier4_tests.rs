// use std::path::PathBuf;
// use tyrus_common::fs::FilePath;

#[test]
fn test_tier4_nestjs_extraction() {
    let current_dir = std::env::current_dir().expect("Failed to get CWD");
    println!("CWD: {:?}", current_dir);
    // CWD is crates/tests (integration_tests), so fixtures are in fixtures/
    let input_path = current_dir.join("fixtures/tier4_nestjs_di/input.ts");

    // Ensure file exists before parsing
    assert!(
        input_path.exists(),
        "Input file does not exist at: {:?}",
        input_path
    );

    let program = tyrus_parser::parse(&input_path).expect("Failed to parse");
    let source_code = std::fs::read_to_string(&input_path).expect("Failed to read source");

    let analysis = tyrus_analyzer::Analyzer::analyze(
        &program,
        source_code,
        input_path.to_string_lossy().to_string(),
    );

    let graph = analysis.graph;

    // Verify Module Extraction
    let cats_module = graph
        .get_module("CatsModule")
        .expect("CatsModule not found in graph");

    assert_eq!(cats_module.name, "CatsModule");

    // Verify Providers
    let provider = cats_module
        .providers
        .iter()
        .find(|p| p.token == "CatsService")
        .expect("CatsService provider not found");

    assert_eq!(provider.token, "CatsService");
    assert_eq!(provider.implementation, "CatsService");

    // Verify Controllers
    assert!(cats_module
        .controllers
        .contains(&"CatsController".to_string()));

    // Verify Injectable Definitions (Implicitly tested by resolve)
    // Let's resolve the graph
    let init_order = graph.resolve().expect("Failed to resolve graph");

    // CatsController depends on CatsService.
    // So CatsService must come before CatsController.
    let service_idx = init_order
        .iter()
        .position(|r| r == "CatsService")
        .expect("CatsService missing");
    let controller_idx = init_order
        .iter()
        .position(|r| r == "CatsController")
        .expect("CatsController missing");

    assert!(
        service_idx < controller_idx,
        "CatsService should be initialized before CatsController"
    );
}

#[test]
fn test_tier4_full_build() {
    let current_dir = std::env::current_dir().expect("Failed to get CWD");
    let fixture_path = current_dir.join("fixtures/tier4_nestjs_di");
    let output_dir = current_dir.join("target/test_output/tier4_full_build");

    // Clean up output dir
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("Failed to clean output dir");
    }
    std::fs::create_dir_all(&output_dir).expect("Failed to create output dir");

    // Run build_project
    // We need to use tyrus_orchestrator::build_project
    // But tyrus_orchestrator is an external crate to integration_tests?
    // integration_tests/Cargo.toml has tyrus_orchestrator dependency.

    tyrus_orchestrator::build_project(&fixture_path, &output_dir).expect("build_project failed");

    // Verify main.rs content
    let main_rs_path = output_dir.join("src/main.rs");
    assert!(main_rs_path.exists(), "main.rs not generated");

    let main_content = std::fs::read_to_string(&main_rs_path).expect("Failed to read main.rs");

    // Check for Service Instantiation
    // allowed both qualified and unqualified depending on implementation
    let has_service = main_content
        .contains("let cats_service = Arc::new(tyrus_app::input::CatsService::new_di());")
        || main_content.contains(
            "let cats_service = std::sync::Arc::new(tyrus_app::input::CatsService::new_di());",
        );

    assert!(
        has_service,
        "CatsService instantiation missing or incorrect: {}",
        main_content
    );

    // Check for Controller Instantiation with Dependency
    let has_controller = main_content.contains("let cats_controller = Arc::new(tyrus_app::input::CatsController::new_di(cats_service.clone()));") ||
                         main_content.contains("let cats_controller = std::sync::Arc::new(tyrus_app::input::CatsController::new_di(cats_service.clone()));");

    assert!(
        has_controller,
        "CatsController instantiation missing or incorrect: {}",
        main_content
    );

    // Check for Router Merge
    assert!(
        main_content.contains("CatsController::router(cats_controller.clone())"),
        "Router merge missing: {main_content}"
    );

    // Check for Extension Layer
    assert!(
        main_content.contains(".layer(Extension(cats_controller.clone()))")
            || main_content.contains(".layer(axum::Extension(cats_controller.clone()))"),
        "Controller extension layer missing"
    );
}

#[test]
fn test_multi_module_build() {
    let current_dir = std::env::current_dir().expect("Failed to get CWD");
    let fixture_path = current_dir.join("fixtures/tier4_multi_module");
    let output_dir = current_dir.join("target/test_output/multi_module_build");

    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("Failed to clean output dir");
    }
    std::fs::create_dir_all(&output_dir).expect("Failed to create output dir");

    tyrus_orchestrator::build_project(&fixture_path, &output_dir).expect("build_project failed");

    // Verify directory structure
    assert!(
        output_dir.join("src/lib.rs").exists(),
        "lib.rs not generated"
    );
    assert!(
        output_dir.join("src/main.rs").exists(),
        "main.rs not generated"
    );
    assert!(
        output_dir.join("src/users").is_dir(),
        "users/ directory not generated"
    );

    // Verify main.rs has both services and controllers
    let main_content =
        std::fs::read_to_string(output_dir.join("src/main.rs")).expect("Failed to read main.rs");

    assert!(
        main_content.contains("UsersService") || main_content.contains("users_service"),
        "UsersService missing from main.rs: {main_content}"
    );
    assert!(
        main_content.contains("AppService") || main_content.contains("app_service"),
        "AppService missing from main.rs: {main_content}"
    );
    assert!(
        main_content.contains("router"),
        "Router setup missing: {main_content}"
    );
}

#[test]
fn test_reference_nestjs_transpiles_and_compiles() {
    let current_dir = std::env::current_dir().expect("Failed to get CWD");
    let fixture_path = current_dir.join("fixtures/reference_nestjs");
    let output_dir = current_dir.join("target/test_output/reference_nestjs_build");

    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("Failed to clean output dir");
    }
    std::fs::create_dir_all(&output_dir).expect("Failed to create output dir");

    // 1. Transpile
    tyrus_orchestrator::build_project(&fixture_path, &output_dir).expect("build_project failed");

    // 2. Verify structure
    assert!(output_dir.join("src/lib.rs").exists(), "lib.rs missing");
    assert!(output_dir.join("src/main.rs").exists(), "main.rs missing");
    assert!(output_dir.join("Cargo.toml").exists(), "Cargo.toml missing");
    assert!(output_dir.join("src/users").is_dir(), "users/ dir missing");

    // 3. Verify compilation
    let output = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(&output_dir)
        .output()
        .expect("Failed to run cargo check");

    assert!(
        output.status.success(),
        "Generated Rust failed to compile:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // 4. Verify main.rs has proper DI and routing
    let main_content =
        std::fs::read_to_string(output_dir.join("src/main.rs")).expect("read main.rs");

    assert!(
        main_content.contains("AppController") && main_content.contains("UsersController"),
        "Both controllers should be in main.rs: {main_content}"
    );
    assert!(
        main_content.contains("router"),
        "Router should be configured: {main_content}"
    );
}

#[test]
fn test_http_equivalence_rust_server() {
    let current_dir = std::env::current_dir().expect("CWD");
    let fixture_path = current_dir.join("fixtures/reference_nestjs");
    let output_dir = current_dir.join("target/test_output/http_equiv_build");

    // 1. Clean + Transpile
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).expect("clean");
    }
    std::fs::create_dir_all(&output_dir).expect("mkdir");
    tyrus_orchestrator::build_project(&fixture_path, &output_dir).expect("transpile");

    // 2. Patch port to 3100 (avoid conflict with user's ports)
    let main_path = output_dir.join("src/main.rs");
    let main_content = std::fs::read_to_string(&main_path).expect("read");
    let patched = main_content.replace("0.0.0.0:3000", "127.0.0.1:3100");
    std::fs::write(&main_path, patched).expect("write");

    // 3. Compile (release for speed)
    let build = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&output_dir)
        .output()
        .expect("cargo build");
    assert!(
        build.status.success(),
        "Build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    // 4. Start server
    let mut server = std::process::Command::new(output_dir.join("target/release/server"))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("start server");

    // 5. Wait for server to be ready
    let ready = wait_for_server("http://127.0.0.1:3100/", 30);
    if !ready {
        server.kill().ok();
        server.wait().ok();
        panic!("Server did not start within 30 attempts");
    }

    // 6. Verify HTTP responses
    verify_endpoint("http://127.0.0.1:3100/", "ok");
    verify_endpoint("http://127.0.0.1:3100/users", "[]");

    // 7. Cleanup
    server.kill().ok();
    server.wait().ok();
}

fn wait_for_server(url: &str, max_attempts: u32) -> bool {
    for _ in 0..max_attempts {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if std::process::Command::new("curl")
            .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", url])
            .output()
            .is_ok_and(|o| o.stdout == b"200")
        {
            return true;
        }
    }
    false
}

fn verify_endpoint(url: &str, expected_contains: &str) {
    let output = std::process::Command::new("curl")
        .args(["-s", url])
        .output()
        .expect("curl failed");
    let body = String::from_utf8_lossy(&output.stdout);
    assert!(
        body.contains(expected_contains),
        "GET {} — expected '{}' in response, got: {}",
        url,
        expected_contains,
        body
    );
}
