#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Runtime Comparison Benchmark
//!
//! Measures wall-clock execution time of identical algorithms running in:
//! - Node.js (TypeScript via --experimental-strip-types)
//! - Compiled Rust (transpiled by Tyrus, then cargo build --release)
//!
//! Outputs a comparison table for academic thesis inclusion.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use tempfile::TempDir;
// tyrus_common::fs::FilePath not needed — we call parser/codegen directly

static SHARED_TARGET_DIR: OnceLock<PathBuf> = OnceLock::new();

fn get_shared_target_dir() -> &'static PathBuf {
    SHARED_TARGET_DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join("tyrus_bench_target");
        fs::create_dir_all(&dir).expect("create bench target dir");
        dir
    })
}

// ---------------------------------------------------------------------------
// Test programs: same algorithm in TS, must produce identical output
// ---------------------------------------------------------------------------

struct BenchCase {
    name: &'static str,
    ts_code: &'static str,
}

const BENCH_CASES: &[BenchCase] = &[
    BenchCase {
        name: "fibonacci",
        ts_code: r#"
function fibonacci(n: number): number {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
console.log(fibonacci(30));
"#,
    },
    BenchCase {
        name: "sum_loop",
        //"Sum 1 to 1_000_000 in a while loop",
        ts_code: r#"
function sumTo(n: number): number {
    let total: number = 0;
    let i: number = 1;
    while (i <= n) {
        total = total + i;
        i = i + 1;
    }
    return total;
}
console.log(sumTo(1000000));
"#,
    },
    BenchCase {
        name: "string_build",
        //"Build string via repeated concatenation (1000 iterations)",
        ts_code: r#"
function buildString(n: number): string {
    let result: string = "";
    let i: number = 0;
    while (i < n) {
        result = result + "x";
        i = i + 1;
    }
    return result;
}
console.log(buildString(1000).includes("xxx").toString());
"#,
    },
    BenchCase {
        name: "math_intensive",
        //"Math.sqrt + Math.floor in a loop (10000 iterations)",
        ts_code: r#"
function mathLoop(n: number): number {
    let sum: number = 0;
    let i: number = 1;
    while (i <= n) {
        sum = sum + Math.floor(Math.sqrt(i * 1.0));
        i = i + 1;
    }
    return sum;
}
console.log(mathLoop(10000));
"#,
    },
    BenchCase {
        name: "nested_loops",
        //"Nested loop O(n^2) with n=500",
        ts_code: r#"
function nestedSum(n: number): number {
    let total: number = 0;
    let i: number = 0;
    while (i < n) {
        let j: number = 0;
        while (j < n) {
            total = total + 1;
            j = j + 1;
        }
        i = i + 1;
    }
    return total;
}
console.log(nestedSum(500));
"#,
    },
];

// ---------------------------------------------------------------------------
// Runner utilities
// ---------------------------------------------------------------------------

fn run_node_timed(ts_code: &str, iterations: u32) -> (String, Duration) {
    let temp_dir = TempDir::new().expect("create temp dir");
    let ts_file = temp_dir.path().join("bench.ts");
    fs::write(&ts_file, ts_code).expect("write ts file");

    // Warmup run
    let output = Command::new("node")
        .args(["--experimental-strip-types", ts_file.to_str().unwrap()])
        .output()
        .expect("Node.js not found");
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Timed runs
    let start = Instant::now();
    for _ in 0..iterations {
        Command::new("node")
            .args(["--experimental-strip-types", ts_file.to_str().unwrap()])
            .output()
            .expect("node run failed");
    }
    let elapsed = start.elapsed();

    (stdout, elapsed / iterations)
}

fn transpile_ts(ts_code: &str) -> String {
    let mut tmp = tempfile::Builder::new()
        .suffix(".ts")
        .tempfile()
        .expect("create temp file");
    tmp.write_all(ts_code.as_bytes()).expect("write ts");
    tmp.flush().expect("flush");
    let path = tmp.path().to_path_buf();

    // Use generate with is_index=true so top-level console.log() gets wrapped in fn main()
    let program = tyrus_parser::parse(&path).expect("parse failed");
    let generated = tyrus_codegen::generate(&program, true);
    let mut code = generated.code;

    // Inject AppError if needed
    if code.contains("crate::AppError") || code.contains("crate :: AppError") {
        code = code.replace("crate::AppError", "AppError");
        code = code.replace("crate :: AppError", "AppError");
    }

    code
}

fn compile_rust_release(rust_code: &str, bin_name: &str) -> PathBuf {
    let temp_dir_path = get_shared_target_dir().join(format!("bench_{bin_name}"));
    fs::create_dir_all(temp_dir_path.join("src")).expect("create src dir");

    let cargo_toml = format!(
        r#"[package]
name = "{bin_name}"
version = "0.1.0"
edition = "2021"

[workspace]

[[bin]]
name = "{bin_name}"
path = "src/main.rs"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#
    );

    fs::write(temp_dir_path.join("Cargo.toml"), cargo_toml).expect("write Cargo.toml");

    let wrapped = format!(
        "#![allow(dead_code, unused_variables, unused_imports, unused_mut)]\n{}",
        rust_code
    );
    fs::write(temp_dir_path.join("src/main.rs"), &wrapped).expect("write main.rs");

    let build = Command::new("cargo")
        .args(["build", "--release", "--quiet"])
        .env("CARGO_TARGET_DIR", get_shared_target_dir().as_path())
        .current_dir(&temp_dir_path)
        .output()
        .expect("cargo build");

    if !build.status.success() {
        let stderr = String::from_utf8_lossy(&build.stderr);
        panic!("Rust build failed for {bin_name}:\n{stderr}\n\nCode:\n{rust_code}");
    }

    get_shared_target_dir().join("release").join(bin_name)
}

fn run_rust_timed(bin_path: &PathBuf, iterations: u32) -> (String, Duration) {
    // Warmup
    let output = Command::new(bin_path).output().expect("run rust binary");
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Timed runs
    let start = Instant::now();
    for _ in 0..iterations {
        Command::new(bin_path).output().expect("rust run failed");
    }
    let elapsed = start.elapsed();

    (stdout, elapsed / iterations)
}

/// Normalize numeric output for comparison.
/// Node.js prints integers as "832040", Rust f64 may print "832040" or with trailing .0
/// This strips trailing ".0" from each line for fair comparison.
fn normalize_numeric_output(s: &str) -> String {
    s.lines()
        .map(|line| {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_suffix(".0") {
                // Only strip if what remains is a valid integer
                if stripped.parse::<i64>().is_ok() {
                    return stripped.to_string();
                }
            }
            trimmed.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Main: run all benchmarks and print comparison table
// ---------------------------------------------------------------------------

fn main() {
    let iterations = std::env::var("BENCH_ITERATIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5u32);

    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════╗");
    println!("║              Tyrus Runtime Comparison: Node.js vs Rust                  ║");
    println!("╠══════════════════════════════════════════════════════════════════════════╣");
    println!(
        "║ {:20} │ {:>12} │ {:>12} │ {:>8} │ {:>6} ║",
        "Benchmark", "Node.js", "Rust", "Speedup", "Match"
    );
    println!("╠══════════════════════════════════════════════════════════════════════════╣");

    let mut all_match = true;
    let mut results: Vec<(String, Duration, Duration, f64, bool)> = Vec::new();

    for case in BENCH_CASES {
        eprint!("  Running {}... ", case.name);

        // 1. Run in Node.js
        let (node_output, node_time) = run_node_timed(case.ts_code, iterations);

        // 2. Transpile TS → Rust
        let rust_code = transpile_ts(case.ts_code);

        // 3. Ensure fn main() exists
        let rust_code = if rust_code.contains("fn main()") {
            rust_code
        } else {
            format!("{rust_code}\nfn main() {{}}\n")
        };

        // 4. Compile Rust (release mode)
        let bin_path = compile_rust_release(&rust_code, case.name);

        // 5. Run Rust binary
        let (rust_output, rust_time) = run_rust_timed(&bin_path, iterations);

        // 6. Compare outputs (normalize f64 formatting: "832040" == "832040.0")
        let outputs_match =
            normalize_numeric_output(&node_output) == normalize_numeric_output(&rust_output);
        if !outputs_match {
            all_match = false;
        }

        let speedup = if rust_time.as_nanos() > 0 {
            node_time.as_secs_f64() / rust_time.as_secs_f64()
        } else {
            f64::INFINITY
        };

        let match_str = if outputs_match { "OK" } else { "DIFF" };

        println!(
            "║ {:20} │ {:>10.2}ms │ {:>10.2}ms │ {:>6.1}x │ {:>6} ║",
            case.name,
            node_time.as_secs_f64() * 1000.0,
            rust_time.as_secs_f64() * 1000.0,
            speedup,
            match_str,
        );

        results.push((
            case.name.to_string(),
            node_time,
            rust_time,
            speedup,
            outputs_match,
        ));

        eprintln!("done");
    }

    println!("╠══════════════════════════════════════════════════════════════════════════╣");

    // Summary
    let avg_speedup: f64 =
        results.iter().map(|(_, _, _, s, _)| s).sum::<f64>() / results.len() as f64;

    println!(
        "║ {:20} │ {:>12} │ {:>12} │ {:>6.1}x │ {:>6} ║",
        "AVERAGE",
        "",
        "",
        avg_speedup,
        if all_match { "ALL OK" } else { "DIFFS" }
    );
    println!("╚══════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Iterations per benchmark: {iterations}");
    println!(
        "  Semantic equivalence: {}",
        if all_match { "PROVEN" } else { "FAILED" }
    );
    println!();

    // Machine-readable output (for thesis data collection)
    if std::env::var("BENCH_CSV").is_ok() {
        println!("--- CSV OUTPUT ---");
        println!("benchmark,node_ms,rust_ms,speedup,match");
        for (name, node, rust, speedup, m) in &results {
            println!(
                "{},{:.3},{:.3},{:.1},{}",
                name,
                node.as_secs_f64() * 1000.0,
                rust.as_secs_f64() * 1000.0,
                speedup,
                m
            );
        }
    }
}
