#[cfg(test)]
mod executable_tests {
    use ox_common::fs::FilePath;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_compile_and_execute_simple_functions() {
        // Transpile TypeScript to Rust
        let ts_path = PathBuf::from("fixtures/executable_simple/input.ts");
        let rust_code =
            ox_orchestrator::build(FilePath::from(ts_path)).expect("Failed to generate Rust code");

        // Create complete executable program
        let program = format!(
            r#"
{}

fn main() {{
    let result1 = add(5.0, 3.0);
    println!("add(5, 3) = {{}}", result1);
    assert_eq!(result1, 8.0);
    
    let result2 = multiply(4.0, 7.0);
    println!("multiply(4, 7) = {{}}", result2);
    assert_eq!(result2, 28.0);
    
    let result3 = calculate_total(10.0, 20.0, 30.0);
    println!("calculate_total(10, 20, 30) = {{}}", result3);
    assert_eq!(result3, 60.0);
    
    println!("✅ All tests passed!");
}}
"#,
            rust_code
        );

        // Compile and execute
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("main.rs");
        fs::write(&src_file, program).unwrap();

        // Compile
        let exe_path = temp_dir.path().join("test_exec");
        let compile = Command::new("rustc")
            .arg("--edition=2021")
            .arg(&src_file)
            .arg("-o")
            .arg(&exe_path)
            .output()
            .expect("Failed to compile");

        assert!(
            compile.status.success(),
            "Compilation failed:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );

        // EXECUTE
        let exec = Command::new(&exe_path).output().expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&exec.stdout);
        println!("Execution output:\n{}", stdout);

        assert!(
            exec.status.success(),
            "Execution failed:\n{}",
            String::from_utf8_lossy(&exec.stderr)
        );

        assert!(stdout.contains("✅ All tests passed!"));
    }

    #[test]
    fn test_compile_and_execute_class() {
        // Transpile class
        let ts_path = PathBuf::from("fixtures/exec_class/input.ts");
        let mut rust_code =
            ox_orchestrator::build(FilePath::from(ts_path)).expect("Failed to generate Rust code");

        // Remove serde derives for standalone compilation
        rust_code = rust_code.replace(", serde :: Serialize, serde :: Deserialize", "");
        rust_code = rust_code.replace("serde :: Serialize, serde :: Deserialize, ", "");
        rust_code = rust_code.replace("serde :: Serialize, serde :: Deserialize", "");

        // Create executable with class instantiation and method calls
        let program = format!(
            r#"
{}

fn main() {{
    let calc = Calculator::new(10.0);
    
    let result1 = calc.add(5.0);
    println!("calc.add(5.0) = {{}}", result1);
    assert_eq!(result1, 15.0, "Expected add to return 15.0");
    
    let result2 = calc.multiply(3.0);
    println!("calc.multiply(3.0) = {{}}", result2);
    assert_eq!(result2, 30.0, "Expected multiply to return 30.0");
    
    let value = calc.getValue();
    println!("calc.getValue() = {{}}", value);
    assert_eq!(value, 10.0, "Expected getValue to return 10.0");
    
    println!("✅ Class execution test passed!");
}}
"#,
            rust_code
        );

        execute_rust_program(&program, "Class execution");
    }

    #[test]
    fn test_compile_and_execute_integration() {
        // Complex integration test combining functions and logic
        let ts_code = r#"
            function square(x: number): number {
                return x * x;
            }
            
            function sumSquares(a: number, b: number): number {
                return square(a) + square(b);
            }
            
            function pythagoras(a: number, b: number): number {
                return sumSquares(a, b);
            }
        "#;

        let temp_dir = std::env::temp_dir();
        let ts_file = temp_dir.join("integration_test.ts");
        std::fs::write(&ts_file, ts_code).unwrap();

        let rust_code =
            ox_orchestrator::build(FilePath::from(ts_file)).expect("Failed to generate Rust code");

        let program = format!(
            r#"
{}

fn main() {{
    let result = pythagoras(3.0, 4.0);
    println!("pythagoras(3, 4) = {{}}", result);
    assert_eq!(result, 25.0, "Expected 3² + 4² = 25");
    
    let result2 = sum_squares(5.0, 12.0);  
    println!("sum_squares(5, 12) = {{}}", result2);
    assert_eq!(result2, 169.0, "Expected 5² + 12² = 169");
    
    println!("✅ Integration test passed!");
}}
"#,
            rust_code
        );

        execute_rust_program(&program, "Integration");
    }

    fn execute_rust_program(program: &str, test_name: &str) {
        {
            let temp_dir = TempDir::new().unwrap();
            let src_file = temp_dir.path().join("main.rs");
            fs::write(&src_file, program).unwrap();

            // Compile
            let exe_path = temp_dir.path().join("test_exec");
            let compile = Command::new("rustc")
                .arg("--edition=2021")
                .arg(&src_file)
                .arg("-o")
                .arg(&exe_path)
                .output()
                .expect("Failed to compile");

            assert!(
                compile.status.success(),
                "{} - Compilation failed:\n{}",
                test_name,
                String::from_utf8_lossy(&compile.stderr)
            );

            // EXECUTE
            let exec = Command::new(&exe_path).output().expect("Failed to execute");

            let stdout = String::from_utf8_lossy(&exec.stdout);
            let stderr = String::from_utf8_lossy(&exec.stderr);

            println!("{} output:", test_name);
            println!("{}", stdout);

            if !stderr.is_empty() {
                {
                    println!("stderr: {}", stderr);
                }
            }

            assert!(
                exec.status.success(),
                "{} - Execution failed:\n{}",
                test_name,
                stderr
            );

            assert!(
                stdout.contains("✅"),
                "{} - Missing success marker",
                test_name
            );
        }
    }
}
