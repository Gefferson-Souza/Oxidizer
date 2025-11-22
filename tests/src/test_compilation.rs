#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;

    /// Test that generated Rust code compiles with rustc
    #[test]
    fn test_rustc_compilation_interface() {
        let ts_code = r#"
            interface User {
                name: string;
                age: number;
            }
        "#;

        verify_rustc_compiles(ts_code, "test_interface");
    }

    #[test]
    fn test_rustc_compilation_class() {
        let ts_code = r#"
            class Dog {
                name: string;
                
                constructor(name: string) {
                    this.name = name;
                }
                
                bark(): number {
                    return 42;
                }
            }
        "#;

        verify_rustc_compiles(ts_code, "test_class");
    }

    #[test]
    fn test_rustc_compilation_function() {
        let ts_code = r#"
            function add(a: number, b: number): number {
                return a + b;
            }
        "#;

        verify_rustc_compiles(ts_code, "test_function");
    }

    fn verify_rustc_compiles(ts_code: &str, test_name: &str) {
        // Write TypeScript to temp file
        let mut ts_file = NamedTempFile::new().unwrap();
        ts_file.write_all(ts_code.as_bytes()).unwrap();

        // Generate Rust code
        let rust_code =
            ox_orchestrator::build(ox_common::fs::FilePath::from(ts_file.path().to_path_buf()))
                .unwrap_or_else(|_| panic!("Failed to generate Rust code for {}", test_name));

        // Write Rust to temp file
        let mut rs_file = NamedTempFile::new().unwrap();
        writeln!(rs_file, "// Auto-generated from {}", test_name).unwrap();
        writeln!(rs_file, "#![allow(dead_code, unused_variables)]").unwrap();
        rs_file.write_all(rust_code.as_bytes()).unwrap();
        rs_file.flush().unwrap();

        // Try to compile with rustc
        let output = Command::new("rustc")
            .arg("--crate-name=oxidizer_test")
            .arg("--crate-type=lib")
            .arg("--edition=2021")
            .arg(rs_file.path())
            .arg("-o")
            .arg("/dev/null")
            .output()
            .expect("Failed to execute rustc");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "rustc compilation failed for {}:\n{}\n\nGenerated Rust code:\n{}",
                test_name, stderr, rust_code
            );
        }
    }
}
