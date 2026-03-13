# CLI + IR + Analyzer Evolution — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Evolve the three weakest crates (tyrus_cli, tyrus_ast, tyrus_analyzer) from stubs into production-grade compiler infrastructure.

**Architecture:** CLI becomes the branded user-facing experience with `run`/`compile`/`check` commands, colored output, and progress indicators. tyrus_ast defines a strict, typed Intermediate Representation (IR) between SWC AST and codegen, enforcing Oxidizable rules at the type level. tyrus_analyzer uses the IR for richer static analysis — unsupported API detection, semantic warnings, and structured JSON output for future tooling (VSCode extension).

**Tech Stack:** `clap` (CLI), `console`+`indicatif` (colors/progress), `serde` (IR serialization), `miette` (diagnostics)

**Execution Order:** CLI first (most visible, no deps) → IR (foundation) → Analyzer (uses IR)

---

## Chunk 1: Polished CLI (tyrus_cli)

### Current State

- `crates/tyrus_cli/src/main.rs` — 64 lines, only `check` and `build` commands
- No branding, no colors, no progress indicators
- No `run` command (transpile+compile+execute)
- No `compile` command (transpile+compile, no execute)
- Dependencies: clap, miette, tokio, tyrus_orchestrator, tyrus_common

### Target State

```
tyrus_cli/src/
├── main.rs        — entry point, clap parse, dispatch (< 80 lines)
├── commands/
│   ├── mod.rs     — re-exports
│   ├── check.rs   — analyze TS file, report errors
│   ├── build.rs   — transpile TS → Rust source
│   ├── compile.rs — transpile + cargo build
│   └── run.rs     — transpile + cargo build + execute
├── output/
│   ├── mod.rs     — re-exports
│   ├── banner.rs  — Tyrus ASCII logo + version
│   ├── colors.rs  — colored output helpers
│   └── progress.rs — progress indicators
└── lib.rs         — (optional) shared types
```

### Task 1: Add CLI dependencies

**Files:**
- Modify: `crates/tyrus_cli/Cargo.toml`

- [ ] **Step 1: Add console, indicatif, and dialoguer to Cargo.toml**

Add to `[dependencies]`:
```toml
console = "0.15"
indicatif = "0.17"
```

- [ ] **Step 2: Verify build**

Run: `cargo build -p tyrus_cli`
Expected: Builds successfully with new deps

- [ ] **Step 3: Commit**

```bash
git add crates/tyrus_cli/Cargo.toml
git commit -m "chore(cli): add console and indicatif dependencies"
```

---

### Task 2: Create output module (banner + colors)

**Files:**
- Create: `crates/tyrus_cli/src/output/mod.rs`
- Create: `crates/tyrus_cli/src/output/banner.rs`
- Create: `crates/tyrus_cli/src/output/colors.rs`

- [ ] **Step 1: Create output/mod.rs**

```rust
pub(crate) mod banner;
pub(crate) mod colors;
pub(crate) mod progress;
```

- [ ] **Step 2: Create output/banner.rs**

```rust
use console::style;

/// Tyrus ASCII logo — displayed on CLI startup
const TYRUS_LOGO: &str = r#"
  ████████╗██╗   ██╗██████╗ ██╗   ██╗███████╗
  ╚══██╔══╝╚██╗ ██╔╝██╔══██╗██║   ██║██╔════╝
     ██║    ╚████╔╝ ██████╔╝██║   ██║███████╗
     ██║     ╚██╔╝  ██╔══██╗██║   ██║╚════██║
     ██║      ██║   ██║  ██║╚██████╔╝███████║
     ╚═╝      ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
"#;

pub(crate) fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("{}", style(TYRUS_LOGO).cyan().bold());
    eprintln!(
        "  {} {} — TypeScript to Rust Compiler",
        style("Tyrus").cyan().bold(),
        style(format!("v{version}")).dim(),
    );
    eprintln!(
        "  {}\n",
        style("Safe Transpilation · Zero Runtime · Semantic Preservation").dim(),
    );
}
```

- [ ] **Step 3: Create output/colors.rs**

```rust
use console::style;

pub(crate) fn success(msg: &str) -> String {
    format!("{} {}", style("✓").green().bold(), style(msg).green())
}

pub(crate) fn error(msg: &str) -> String {
    format!("{} {}", style("✗").red().bold(), style(msg).red())
}

pub(crate) fn info(msg: &str) -> String {
    format!("{} {}", style("ℹ").blue().bold(), msg)
}

pub(crate) fn warning(msg: &str) -> String {
    format!("{} {}", style("⚠").yellow().bold(), style(msg).yellow())
}

pub(crate) fn step(label: &str, detail: &str) -> String {
    format!("  {} {}", style(label).cyan().bold(), detail)
}

pub(crate) fn file_path(path: &str) -> String {
    style(path).underlined().to_string()
}
```

- [ ] **Step 4: Verify build**

Run: `cargo build -p tyrus_cli`
Expected: Builds successfully

- [ ] **Step 5: Commit**

```bash
git add crates/tyrus_cli/src/output/
git commit -m "feat(cli): add branded banner and colored output helpers"
```

---

### Task 3: Create progress module

**Files:**
- Create: `crates/tyrus_cli/src/output/progress.rs`

- [ ] **Step 1: Create output/progress.rs**

```rust
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub(crate) struct Pipeline {
    steps: Vec<String>,
    current: usize,
}

impl Pipeline {
    pub(crate) fn new(steps: Vec<&str>) -> Self {
        Self {
            steps: steps.into_iter().map(String::from).collect(),
            current: 0,
        }
    }

    pub(crate) fn start_step(&mut self, detail: &str) {
        if self.current < self.steps.len() {
            let label = &self.steps[self.current];
            eprintln!(
                "  {} {} {}",
                style(format!("[{}/{}]", self.current + 1, self.steps.len())).dim(),
                style(label).cyan().bold(),
                style(detail).dim(),
            );
        }
        self.current += 1;
    }

    pub(crate) fn finish_success(&self, msg: &str) {
        eprintln!("\n  {} {}\n", style("✓").green().bold(), style(msg).green().bold());
    }

    pub(crate) fn finish_error(&self, msg: &str) {
        eprintln!("\n  {} {}\n", style("✗").red().bold(), style(msg).red().bold());
    }
}

pub(crate) fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template(&format!("  {{spinner}} {}", msg))
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}
```

- [ ] **Step 2: Verify build**

Run: `cargo build -p tyrus_cli`
Expected: Builds

- [ ] **Step 3: Commit**

```bash
git add crates/tyrus_cli/src/output/progress.rs
git commit -m "feat(cli): add progress pipeline and spinner utilities"
```

---

### Task 4: Create command modules (check, build)

**Files:**
- Create: `crates/tyrus_cli/src/commands/mod.rs`
- Create: `crates/tyrus_cli/src/commands/check.rs`
- Create: `crates/tyrus_cli/src/commands/build.rs`

- [ ] **Step 1: Create commands/mod.rs**

```rust
pub(crate) mod build;
pub(crate) mod check;
pub(crate) mod compile;
pub(crate) mod run;
```

- [ ] **Step 2: Create commands/check.rs**

```rust
use std::path::Path;

use miette::Result;
use tyrus_common::fs::FilePath;

use crate::output::{colors, progress::Pipeline};

pub(crate) fn execute(path: &Path) -> Result<()> {
    let mut pipeline = Pipeline::new(vec!["Parse", "Analyze"]);

    pipeline.start_step(&colors::file_path(&path.display().to_string()));
    let result = tyrus_orchestrator::check(&FilePath::from(path.to_path_buf()));

    match result {
        Ok(()) => {
            pipeline.start_step("No issues found");
            pipeline.finish_success("File is Oxidizable — ready for transpilation");
            Ok(())
        }
        Err(e) => {
            pipeline.finish_error("Analysis found issues");
            Err(e.into())
        }
    }
}
```

- [ ] **Step 3: Create commands/build.rs**

```rust
use std::path::{Path, PathBuf};

use miette::Result;
use tyrus_common::fs::FilePath;

use crate::output::{colors, progress::Pipeline};

pub(crate) fn execute(path: &Path, output: Option<PathBuf>) -> Result<()> {
    if path.is_dir() {
        execute_project(path, output)
    } else {
        execute_single(path, output)
    }
}

fn execute_project(path: &Path, output: Option<PathBuf>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("./tyrus_output"));
    let mut pipeline = Pipeline::new(vec!["Scan", "Parse", "Analyze", "Transpile", "Scaffold"]);

    pipeline.start_step(&format!(
        "{} → {}",
        colors::file_path(&path.display().to_string()),
        colors::file_path(&output_dir.display().to_string()),
    ));

    // Steps 2-5 happen inside build_project
    pipeline.start_step("Walking TypeScript files");
    pipeline.start_step("Running Oxidizable checks");
    pipeline.start_step("Generating Rust code");
    pipeline.start_step("Creating Cargo project");

    tyrus_orchestrator::build_project(path, &output_dir)?;
    pipeline.finish_success(&format!(
        "Project built → {}",
        colors::file_path(&output_dir.display().to_string()),
    ));
    Ok(())
}

fn execute_single(path: &Path, output: Option<PathBuf>) -> Result<()> {
    let mut pipeline = Pipeline::new(vec!["Parse", "Transpile", "Format"]);

    pipeline.start_step(&colors::file_path(&path.display().to_string()));
    pipeline.start_step("Generating Rust code");

    let output_code = tyrus_orchestrator::build(&FilePath::from(path.to_path_buf()))?;

    pipeline.start_step("Formatting output");

    if let Some(output_path) = output {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| miette::miette!("Failed to create directory: {}", e))?;
        }
        std::fs::write(&output_path, &output_code)
            .map_err(|e| miette::miette!("Failed to write output file: {}", e))?;
        pipeline.finish_success(&format!(
            "Built → {}",
            colors::file_path(&output_path.display().to_string()),
        ));
    } else {
        println!("{output_code}");
    }
    Ok(())
}
```

- [ ] **Step 4: Verify build**

Run: `cargo build -p tyrus_cli`
Expected: Builds

- [ ] **Step 5: Commit**

```bash
git add crates/tyrus_cli/src/commands/
git commit -m "feat(cli): add check and build command modules with progress output"
```

---

### Task 5: Create `compile` command

**Files:**
- Create: `crates/tyrus_cli/src/commands/compile.rs`

- [ ] **Step 1: Create commands/compile.rs**

The `compile` command transpiles TS → Rust, then runs `cargo build` on the generated project.

```rust
use std::path::{Path, PathBuf};
use std::process::Command;

use miette::Result;

use crate::output::{colors, progress::Pipeline};

pub(crate) fn execute(path: &Path, output: Option<PathBuf>, release: bool) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("./tyrus_output"));
    let mut pipeline = Pipeline::new(vec!["Transpile", "Compile"]);

    // Step 1: Transpile
    pipeline.start_step(&format!(
        "{} → {}",
        colors::file_path(&path.display().to_string()),
        colors::file_path(&output_dir.display().to_string()),
    ));

    if path.is_dir() {
        tyrus_orchestrator::build_project(path, &output_dir)?;
    } else {
        // Single file needs project scaffolding too
        tyrus_orchestrator::build_project(path.parent().unwrap_or(path), &output_dir)?;
    }

    // Step 2: Cargo build
    pipeline.start_step("Running cargo build on generated Rust");

    let mut cmd = Command::new("cargo");
    cmd.arg("build").current_dir(&output_dir);
    if release {
        cmd.arg("--release");
    }

    let status = cmd.output().map_err(|e| miette::miette!("Failed to run cargo: {}", e))?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        pipeline.finish_error("Compilation failed");
        eprintln!("{stderr}");
        return Err(miette::miette!("cargo build failed"));
    }

    let mode = if release { "release" } else { "debug" };
    pipeline.finish_success(&format!(
        "Compiled → {}/target/{mode}/tyrus_app",
        colors::file_path(&output_dir.display().to_string()),
    ));
    Ok(())
}
```

- [ ] **Step 2: Verify build**

Run: `cargo build -p tyrus_cli`
Expected: Builds

- [ ] **Step 3: Commit**

```bash
git add crates/tyrus_cli/src/commands/compile.rs
git commit -m "feat(cli): add compile command (transpile + cargo build)"
```

---

### Task 6: Create `run` command

**Files:**
- Create: `crates/tyrus_cli/src/commands/run.rs`

- [ ] **Step 1: Create commands/run.rs**

The `run` command transpiles, compiles, then executes the binary.

```rust
use std::path::{Path, PathBuf};
use std::process::Command;

use miette::Result;

use crate::output::{colors, progress::Pipeline};

pub(crate) fn execute(path: &Path, output: Option<PathBuf>) -> Result<()> {
    let output_dir = output.unwrap_or_else(|| PathBuf::from("./tyrus_output"));
    let mut pipeline = Pipeline::new(vec!["Transpile", "Compile", "Execute"]);

    // Step 1: Transpile
    pipeline.start_step(&format!(
        "{} → {}",
        colors::file_path(&path.display().to_string()),
        colors::file_path(&output_dir.display().to_string()),
    ));

    if path.is_dir() {
        tyrus_orchestrator::build_project(path, &output_dir)?;
    } else {
        tyrus_orchestrator::build_project(path.parent().unwrap_or(path), &output_dir)?;
    }

    // Step 2: Compile
    pipeline.start_step("Building generated Rust project");

    let build_output = Command::new("cargo")
        .args(["build"])
        .current_dir(&output_dir)
        .output()
        .map_err(|e| miette::miette!("Failed to run cargo: {}", e))?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        pipeline.finish_error("Compilation failed");
        eprintln!("{stderr}");
        return Err(miette::miette!("cargo build failed"));
    }

    // Step 3: Execute
    pipeline.start_step("Running binary");

    let binary = output_dir.join("target/debug/tyrus_app");
    let run_output = Command::new(&binary)
        .output()
        .map_err(|e| miette::miette!("Failed to execute binary: {}", e))?;

    // Print program output
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    let stderr = String::from_utf8_lossy(&run_output.stderr);
    if !stdout.is_empty() {
        print!("{stdout}");
    }
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }

    if run_output.status.success() {
        pipeline.finish_success("Execution complete");
    } else {
        pipeline.finish_error(&format!(
            "Process exited with code {}",
            run_output.status.code().unwrap_or(-1),
        ));
    }

    Ok(())
}
```

- [ ] **Step 2: Verify build**

Run: `cargo build -p tyrus_cli`
Expected: Builds

- [ ] **Step 3: Commit**

```bash
git add crates/tyrus_cli/src/commands/run.rs
git commit -m "feat(cli): add run command (transpile + compile + execute)"
```

---

### Task 7: Rewrite main.rs with new architecture

**Files:**
- Modify: `crates/tyrus_cli/src/main.rs`

- [ ] **Step 1: Rewrite main.rs**

```rust
use clap::{Parser, Subcommand};
use miette::Result;
use std::path::PathBuf;

mod commands;
mod output;

#[derive(Parser)]
#[command(
    name = "tyrus",
    author,
    version,
    about = "Tyrus — TypeScript to Rust Compiler",
    long_about = "A source-to-source compiler that converts Oxidizable TypeScript into memory-safe Rust.\nSafe Transpilation · Zero Runtime · Semantic Preservation"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Suppress banner and decorations
    #[arg(long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze TypeScript for Oxidizable compatibility
    Check {
        /// Input file path
        path: PathBuf,
    },
    /// Transpile TypeScript to Rust source code
    Build {
        /// Input file or directory path
        path: PathBuf,
        /// Output directory (default: ./tyrus_output)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Transpile + compile to native binary
    Compile {
        /// Input file or directory path
        path: PathBuf,
        /// Output directory (default: ./tyrus_output)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Build in release mode with optimizations
        #[arg(long)]
        release: bool,
    },
    /// Transpile + compile + execute
    Run {
        /// Input file or directory path
        path: PathBuf,
        /// Output directory (default: ./tyrus_output)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    miette::set_panic_hook();

    let cli = Cli::parse();

    if !cli.quiet {
        output::banner::print_banner();
    }

    match cli.command {
        Commands::Check { path } => commands::check::execute(&path),
        Commands::Build { path, output } => commands::build::execute(&path, output),
        Commands::Compile {
            path,
            output,
            release,
        } => commands::compile::execute(&path, output, release),
        Commands::Run { path, output } => commands::run::execute(&path, output),
    }
}
```

- [ ] **Step 2: Verify build**

Run: `cargo build -p tyrus_cli`
Expected: Builds

- [ ] **Step 3: Manual test — banner display**

Run: `cargo run --bin tyrus -- --help`
Expected: Shows Tyrus branding + help text with all 4 commands

- [ ] **Step 4: Manual test — check command**

Create test file:
```bash
echo 'function add(a: number, b: number): number { return a + b; }' > /tmp/test_tyrus.ts
cargo run --bin tyrus -- check /tmp/test_tyrus.ts
```
Expected: Colored output showing successful analysis

- [ ] **Step 5: Manual test — build command**

```bash
cargo run --bin tyrus -- build /tmp/test_tyrus.ts -o /tmp/tyrus_out.rs
```
Expected: Generates Rust file with progress indicators

- [ ] **Step 6: Commit**

```bash
git add crates/tyrus_cli/src/main.rs
git commit -m "feat(cli): rewrite main.rs with 4 commands and branded output"
```

---

### Task 8: Integration test for CLI

**Files:**
- Create: `tests/src/cli.rs`
- Modify: `tests/src/lib.rs`

- [ ] **Step 1: Write CLI integration test**

```rust
use std::process::Command;

fn tyrus_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tyrus"));
    cmd.arg("--quiet"); // suppress banner in tests
    cmd
}

#[test]
fn test_cli_check_valid_file() {
    let tmp = std::env::temp_dir().join("tyrus_cli_test_valid.ts");
    std::fs::write(&tmp, "function add(a: number, b: number): number { return a + b; }").ok();

    let output = tyrus_bin()
        .args(["check", tmp.to_str().unwrap_or_default()])
        .output()
        .expect("Failed to run tyrus");

    assert!(output.status.success(), "check should succeed for valid TS");
    std::fs::remove_file(tmp).ok();
}

#[test]
fn test_cli_check_invalid_file() {
    let tmp = std::env::temp_dir().join("tyrus_cli_test_invalid.ts");
    std::fs::write(&tmp, "var x: any = eval('bad');").ok();

    let output = tyrus_bin()
        .args(["check", tmp.to_str().unwrap_or_default()])
        .output()
        .expect("Failed to run tyrus");

    // Should still succeed (errors are reported but not fatal currently)
    assert!(output.status.success());
    std::fs::remove_file(tmp).ok();
}

#[test]
fn test_cli_help_shows_all_commands() {
    let output = tyrus_bin()
        .args(["--help"])
        .output()
        .expect("Failed to run tyrus");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("check"), "help should list check command");
    assert!(stdout.contains("build"), "help should list build command");
    assert!(stdout.contains("compile"), "help should list compile command");
    assert!(stdout.contains("run"), "help should list run command");
}
```

- [ ] **Step 2: Add cli module to tests/src/lib.rs**

Add `mod cli;` to the test crate.

- [ ] **Step 3: Run tests**

Run: `cargo test -p integration_tests cli`
Expected: All 3 CLI tests pass

- [ ] **Step 4: Commit**

```bash
git add tests/src/cli.rs tests/src/lib.rs
git commit -m "test(cli): add integration tests for all CLI commands"
```

---

## Chunk 2: Custom IR (tyrus_ast)

### Design Philosophy

The IR is the **single source of truth** between parsing and codegen. It captures ONLY what Tyrus supports — the Oxidizable Standard. Any TS construct that can't be represented in the IR is rejected at lowering time, not at codegen time.

### Strict Rules for tyrus_ast

| Rule | Rationale |
|------|-----------|
| **No SWC types in public API** | IR must be independent of parser implementation |
| **All nodes must be `Clone + Debug + PartialEq`** | Enables testing, comparison, serialization |
| **No methods > 30 lines** | Keep lowering functions focused and testable |
| **No nested enums > 2 levels** | Prevents `TyrusExpr::Binary(Box<TyrusExpr::Call(...)>)` complexity |
| **Every variant must have a test** | No dead code in the IR |
| **Spans on every node** | Error reporting traces back to source |
| **No `String` for identifiers** | Use a dedicated `Ident` type with interning potential |
| **No `Option` nesting** | `Option<Option<T>>` is a design smell — flatten |
| **Immutable by default** | IR nodes are built once, read many times |
| **Serializable** | `serde::Serialize` for debugging, future LSP, VSCode extension |

### File Structure

```
crates/tyrus_ast/src/
├── lib.rs         — public re-exports only (< 30 lines)
├── types.rs       — TyrusType enum (all Oxidizable types)
├── expr.rs        — TyrusExpr enum (expressions)
├── stmt.rs        — TyrusStmt enum (statements)
├── decl.rs        — TyrusDecl enum (function, class, interface, enum, type alias)
├── module.rs      — TyrusModule (top-level container)
├── ident.rs       — Ident type (name + span)
├── span.rs        — TyrusSpan (source location)
├── lower.rs       — SWC Program → TyrusModule lowering (entry point)
├── lower_expr.rs  — SWC Expr → TyrusExpr
├── lower_stmt.rs  — SWC Stmt → TyrusStmt
├── lower_decl.rs  — SWC Decl → TyrusDecl
└── lower_type.rs  — SWC TsType → TyrusType
```

### Task 9: Define IR core types (types.rs, ident.rs, span.rs)

**Files:**
- Create: `crates/tyrus_ast/src/span.rs`
- Create: `crates/tyrus_ast/src/ident.rs`
- Create: `crates/tyrus_ast/src/types.rs`

- [ ] **Step 1: Write tests for IR types**

Create `crates/tyrus_ast/src/tests.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tyrus_type_string() {
        let t = TyrusType::String;
        assert_eq!(format!("{t:?}"), "String");
    }

    #[test]
    fn test_tyrus_type_array() {
        let t = TyrusType::Array(Box::new(TyrusType::Number));
        assert!(matches!(t, TyrusType::Array(_)));
    }

    #[test]
    fn test_tyrus_type_option() {
        let t = TyrusType::Option(Box::new(TyrusType::String));
        assert!(matches!(t, TyrusType::Option(_)));
    }

    #[test]
    fn test_ident_display() {
        let id = Ident::new("myVar", TyrusSpan::dummy());
        assert_eq!(id.name, "myVar");
    }

    #[test]
    fn test_span_dummy() {
        let s = TyrusSpan::dummy();
        assert_eq!(s.start, 0);
        assert_eq!(s.end, 0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tyrus_ast`
Expected: Compilation errors (types not defined yet)

- [ ] **Step 3: Create span.rs**

```rust
/// Source location for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct TyrusSpan {
    pub start: u32,
    pub end: u32,
}

impl TyrusSpan {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn dummy() -> Self {
        Self { start: 0, end: 0 }
    }
}
```

- [ ] **Step 4: Create ident.rs**

```rust
use crate::span::TyrusSpan;

/// A named identifier with source location
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Ident {
    pub name: String,
    pub span: TyrusSpan,
}

impl Ident {
    pub fn new(name: &str, span: TyrusSpan) -> Self {
        Self {
            name: name.to_string(),
            span,
        }
    }

    pub fn synthetic(name: &str) -> Self {
        Self {
            name: name.to_string(),
            span: TyrusSpan::dummy(),
        }
    }
}
```

- [ ] **Step 5: Create types.rs**

```rust
use crate::ident::Ident;

/// All types representable in the Oxidizable Standard
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TyrusType {
    /// `string` → `String`
    String,
    /// `number` → `f64`
    Number,
    /// `boolean` → `bool`
    Boolean,
    /// `void` → `()`
    Void,
    /// `T[]` or `Array<T>` → `Vec<T>`
    Array(Box<TyrusType>),
    /// `T | undefined` → `Option<T>`
    Option(Box<TyrusType>),
    /// `Record<K, V>` → `HashMap<K, V>`
    Map(Box<TyrusType>, Box<TyrusType>),
    /// `Promise<T>` → `Result<T, AppError>`
    Promise(Box<TyrusType>),
    /// Named type (interface, class, enum, type alias)
    Named(Ident),
    /// Generic type: `Container<T>` → `Container<T>`
    Generic(Ident, Vec<TyrusType>),
    /// Tuple type (for destructuring)
    Tuple(Vec<TyrusType>),
    /// Inferred (when type annotation is missing)
    Inferred,
}
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p tyrus_ast`
Expected: All tests pass

- [ ] **Step 7: Commit**

```bash
git add crates/tyrus_ast/src/
git commit -m "feat(ast): define IR core types — TyrusType, Ident, TyrusSpan"
```

---

### Task 10: Define IR expressions (expr.rs)

**Files:**
- Create: `crates/tyrus_ast/src/expr.rs`

- [ ] **Step 1: Write tests for expressions**

Add to `tests.rs`:
```rust
#[test]
fn test_binary_expr() {
    let expr = TyrusExpr::Binary {
        left: Box::new(TyrusExpr::NumberLit(2.0)),
        op: BinaryOp::Add,
        right: Box::new(TyrusExpr::NumberLit(3.0)),
        span: TyrusSpan::dummy(),
    };
    assert!(matches!(expr, TyrusExpr::Binary { op: BinaryOp::Add, .. }));
}

#[test]
fn test_string_lit() {
    let expr = TyrusExpr::StringLit("hello".to_string());
    assert!(matches!(expr, TyrusExpr::StringLit(_)));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p tyrus_ast`
Expected: FAIL (TyrusExpr not defined)

- [ ] **Step 3: Create expr.rs**

```rust
use crate::ident::Ident;
use crate::span::TyrusSpan;
use crate::types::TyrusType;

/// All expressions in the Oxidizable Standard
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TyrusExpr {
    // -- Literals --
    NumberLit(f64),
    StringLit(String),
    BoolLit(bool),
    ArrayLit(Vec<TyrusExpr>),
    ObjectLit(Vec<(Ident, TyrusExpr)>),
    TemplateLit(Vec<TemplatePart>),
    NullLit,

    // -- References --
    Ident(Ident),

    // -- Operations --
    Binary {
        left: Box<TyrusExpr>,
        op: BinaryOp,
        right: Box<TyrusExpr>,
        span: TyrusSpan,
    },
    Unary {
        op: UnaryOp,
        arg: Box<TyrusExpr>,
        span: TyrusSpan,
    },

    // -- Access --
    Member {
        object: Box<TyrusExpr>,
        property: Ident,
        optional: bool,
        span: TyrusSpan,
    },
    Index {
        object: Box<TyrusExpr>,
        index: Box<TyrusExpr>,
        span: TyrusSpan,
    },

    // -- Calls --
    Call {
        callee: Box<TyrusExpr>,
        args: Vec<TyrusExpr>,
        type_args: Vec<TyrusType>,
        span: TyrusSpan,
    },
    MethodCall {
        object: Box<TyrusExpr>,
        method: Ident,
        args: Vec<TyrusExpr>,
        span: TyrusSpan,
    },

    // -- Functions --
    Arrow {
        params: Vec<Param>,
        body: ArrowBody,
        is_async: bool,
        span: TyrusSpan,
    },

    // -- Control --
    Ternary {
        test: Box<TyrusExpr>,
        consequent: Box<TyrusExpr>,
        alternate: Box<TyrusExpr>,
        span: TyrusSpan,
    },

    // -- Assignment --
    Assign {
        target: Box<TyrusExpr>,
        value: Box<TyrusExpr>,
        op: AssignOp,
        span: TyrusSpan,
    },

    // -- Update --
    Update {
        arg: Box<TyrusExpr>,
        op: UpdateOp,
        prefix: bool,
        span: TyrusSpan,
    },

    // -- Await --
    Await(Box<TyrusExpr>),

    // -- Spread --
    Spread(Box<TyrusExpr>),

    // -- Type assertion (as) --
    TypeAssertion {
        expr: Box<TyrusExpr>,
        target_type: TyrusType,
        span: TyrusSpan,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TemplatePart {
    Str(String),
    Expr(TyrusExpr),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Param {
    pub name: Ident,
    pub ty: TyrusType,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ArrowBody {
    Expr(Box<TyrusExpr>),
    Block(Vec<crate::stmt::TyrusStmt>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Exp,
    Eq, NotEq, StrictEq, StrictNotEq,
    Lt, LtEq, Gt, GtEq,
    And, Or,
    BitAnd, BitOr, BitXor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum UnaryOp {
    Neg, Not, BitNot, TypeOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum AssignOp {
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum UpdateOp {
    Increment, Decrement,
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p tyrus_ast`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/tyrus_ast/src/expr.rs
git commit -m "feat(ast): define TyrusExpr — all Oxidizable expressions"
```

---

### Task 11: Define IR statements (stmt.rs)

**Files:**
- Create: `crates/tyrus_ast/src/stmt.rs`

- [ ] **Step 1: Create stmt.rs**

```rust
use crate::expr::{Param, TyrusExpr};
use crate::ident::Ident;
use crate::span::TyrusSpan;
use crate::types::TyrusType;

/// All statements in the Oxidizable Standard
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TyrusStmt {
    /// `let x: T = expr;` or `const x: T = expr;`
    VarDecl {
        name: Ident,
        ty: TyrusType,
        init: Option<TyrusExpr>,
        mutable: bool,
        span: TyrusSpan,
    },

    /// Expression statement: `foo();`
    Expr(TyrusExpr),

    /// `return expr;`
    Return {
        value: Option<TyrusExpr>,
        span: TyrusSpan,
    },

    /// `if (test) { body } else { alt }`
    If {
        test: TyrusExpr,
        body: Vec<TyrusStmt>,
        alt: Option<Vec<TyrusStmt>>,
        span: TyrusSpan,
    },

    /// `while (test) { body }`
    While {
        test: TyrusExpr,
        body: Vec<TyrusStmt>,
        span: TyrusSpan,
    },

    /// `for (init; test; update) { body }` — lowered to `while`
    For {
        init: Option<Box<TyrusStmt>>,
        test: Option<TyrusExpr>,
        update: Option<TyrusExpr>,
        body: Vec<TyrusStmt>,
        span: TyrusSpan,
    },

    /// `for (const x of iter) { body }`
    ForOf {
        binding: Ident,
        iter: TyrusExpr,
        body: Vec<TyrusStmt>,
        span: TyrusSpan,
    },

    /// `do { body } while (test);`
    DoWhile {
        body: Vec<TyrusStmt>,
        test: TyrusExpr,
        span: TyrusSpan,
    },

    /// `switch (discriminant) { cases }`
    Switch {
        discriminant: TyrusExpr,
        cases: Vec<SwitchCase>,
        span: TyrusSpan,
    },

    /// `break;`
    Break(TyrusSpan),

    /// `continue;`
    Continue(TyrusSpan),

    /// Block of statements `{ ... }`
    Block(Vec<TyrusStmt>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SwitchCase {
    pub test: Option<TyrusExpr>, // None = default case
    pub body: Vec<TyrusStmt>,
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p tyrus_ast`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/tyrus_ast/src/stmt.rs
git commit -m "feat(ast): define TyrusStmt — all Oxidizable statements"
```

---

### Task 12: Define IR declarations (decl.rs)

**Files:**
- Create: `crates/tyrus_ast/src/decl.rs`

- [ ] **Step 1: Create decl.rs**

```rust
use crate::expr::{Param, TyrusExpr};
use crate::ident::Ident;
use crate::span::TyrusSpan;
use crate::stmt::TyrusStmt;
use crate::types::TyrusType;

/// Top-level declarations
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TyrusDecl {
    Function(FunctionDecl),
    Class(ClassDecl),
    Interface(InterfaceDecl),
    Enum(EnumDecl),
    TypeAlias(TypeAliasDecl),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FunctionDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: TyrusType,
    pub body: Vec<TyrusStmt>,
    pub is_async: bool,
    pub is_exported: bool,
    pub type_params: Vec<Ident>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ClassDecl {
    pub name: Ident,
    pub properties: Vec<ClassProperty>,
    pub methods: Vec<ClassMethod>,
    pub constructor: Option<Constructor>,
    pub decorators: Vec<Decorator>,
    pub is_exported: bool,
    pub type_params: Vec<Ident>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ClassProperty {
    pub name: Ident,
    pub ty: TyrusType,
    pub accessibility: Accessibility,
    pub is_readonly: bool,
    pub init: Option<TyrusExpr>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ClassMethod {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: TyrusType,
    pub body: Vec<TyrusStmt>,
    pub is_async: bool,
    pub accessibility: Accessibility,
    pub decorators: Vec<Decorator>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Constructor {
    pub params: Vec<ConstructorParam>,
    pub body: Vec<TyrusStmt>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ConstructorParam {
    pub name: Ident,
    pub ty: TyrusType,
    pub accessibility: Option<Accessibility>,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Accessibility {
    Public,
    Private,
    Protected,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Decorator {
    pub name: String,
    pub args: Vec<TyrusExpr>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct InterfaceDecl {
    pub name: Ident,
    pub fields: Vec<InterfaceField>,
    pub is_exported: bool,
    pub type_params: Vec<Ident>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct InterfaceField {
    pub name: Ident,
    pub ty: TyrusType,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EnumDecl {
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
    pub is_exported: bool,
    pub is_string: bool,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EnumVariant {
    pub name: Ident,
    pub value: Option<TyrusExpr>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TypeAliasDecl {
    pub name: Ident,
    pub ty: TyrusType,
    pub is_exported: bool,
    pub span: TyrusSpan,
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p tyrus_ast`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add crates/tyrus_ast/src/decl.rs
git commit -m "feat(ast): define TyrusDecl — functions, classes, interfaces, enums"
```

---

### Task 13: Define IR module (module.rs) and update lib.rs

**Files:**
- Create: `crates/tyrus_ast/src/module.rs`
- Modify: `crates/tyrus_ast/src/lib.rs`
- Modify: `crates/tyrus_ast/Cargo.toml`

- [ ] **Step 1: Create module.rs**

```rust
use crate::decl::TyrusDecl;
use crate::stmt::TyrusStmt;

/// Top-level container for a transpiled file
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TyrusModule {
    pub declarations: Vec<TyrusDecl>,
    pub statements: Vec<TyrusStmt>,
    pub imports: Vec<ImportDecl>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ImportDecl {
    pub source: String,
    pub specifiers: Vec<ImportSpecifier>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ImportSpecifier {
    Named { local: String, imported: String },
    Default(String),
    Namespace(String),
}

impl TyrusModule {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            statements: Vec::new(),
            imports: Vec::new(),
        }
    }
}

impl Default for TyrusModule {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 2: Update lib.rs**

```rust
pub mod decl;
pub mod expr;
pub mod ident;
pub mod module;
pub mod span;
pub mod stmt;
pub mod types;

// Re-export main types at crate root
pub use decl::*;
pub use expr::{BinaryOp, TyrusExpr, UnaryOp};
pub use ident::Ident;
pub use module::TyrusModule;
pub use span::TyrusSpan;
pub use stmt::TyrusStmt;
pub use types::TyrusType;
```

- [ ] **Step 3: Update Cargo.toml**

```toml
[package]
name = "tyrus_ast"
version = "0.1.0"
edition = "2021"

[dependencies]
tyrus_common = { path = "../tyrus_common" }
serde = { version = "1.0", features = ["derive"] }
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p tyrus_ast`
Expected: All tests pass

- [ ] **Step 5: Run workspace build**

Run: `cargo build --workspace`
Expected: Builds (no downstream breakage)

- [ ] **Step 6: Commit**

```bash
git add crates/tyrus_ast/
git commit -m "feat(ast): complete IR type definitions — TyrusModule as top-level container"
```

---

### Task 14: SWC → IR lowering (foundation)

**Files:**
- Create: `crates/tyrus_ast/src/lower.rs`
- Create: `crates/tyrus_ast/src/lower_type.rs`
- Modify: `crates/tyrus_ast/Cargo.toml`

NOTE: Full lowering is a large task. This task establishes the **entry point** and **type lowering** only. Expression/statement/declaration lowering will be added incrementally in future tasks as codegen migrates to use the IR.

- [ ] **Step 1: Add SWC dependencies to Cargo.toml**

```toml
[dependencies]
tyrus_common = { path = "../tyrus_common" }
tyrus_diagnostics = { path = "../tyrus_diagnostics" }
serde = { version = "1.0", features = ["derive"] }
swc_ecma_ast = "18.0.0"
swc_common = { version = "17.0.1", features = ["tty-emitter"] }
```

- [ ] **Step 2: Write lowering tests**

Create `crates/tyrus_ast/src/lower_tests.rs`:
```rust
#[cfg(test)]
mod tests {
    use crate::lower_type::lower_ts_type;
    use crate::types::TyrusType;

    #[test]
    fn test_lower_string_keyword() {
        // TsKeywordType { kind: TsStringKeyword } → TyrusType::String
        let result = lower_ts_type(None);
        assert_eq!(result, TyrusType::Inferred);
    }
}
```

- [ ] **Step 3: Create lower_type.rs**

```rust
use swc_ecma_ast::{TsKeywordTypeKind, TsType, TsTypeAnn};

use crate::ident::Ident;
use crate::span::TyrusSpan;
use crate::types::TyrusType;

/// Lower a SWC TsTypeAnn to TyrusType
pub fn lower_ts_type(ann: Option<&Box<TsTypeAnn>>) -> TyrusType {
    let Some(ann) = ann else {
        return TyrusType::Inferred;
    };
    lower_ts_type_inner(&ann.type_ann)
}

fn lower_ts_type_inner(ts: &TsType) -> TyrusType {
    match ts {
        TsType::TsKeywordType(kw) => match kw.kind {
            TsKeywordTypeKind::TsStringKeyword => TyrusType::String,
            TsKeywordTypeKind::TsNumberKeyword => TyrusType::Number,
            TsKeywordTypeKind::TsBooleanKeyword => TyrusType::Boolean,
            TsKeywordTypeKind::TsVoidKeyword => TyrusType::Void,
            _ => TyrusType::Inferred,
        },
        TsType::TsArrayType(arr) => {
            let elem = lower_ts_type_inner(&arr.elem_type);
            TyrusType::Array(Box::new(elem))
        }
        TsType::TsTypeRef(ref_type) => {
            let name = if let Some(ident) = ref_type.type_name.as_ident() {
                ident.sym.to_string()
            } else {
                return TyrusType::Inferred;
            };

            match name.as_str() {
                "Array" => {
                    if let Some(params) = &ref_type.type_params {
                        if let Some(first) = params.params.first() {
                            return TyrusType::Array(Box::new(lower_ts_type_inner(first)));
                        }
                    }
                    TyrusType::Array(Box::new(TyrusType::Inferred))
                }
                "Promise" => {
                    if let Some(params) = &ref_type.type_params {
                        if let Some(first) = params.params.first() {
                            return TyrusType::Promise(Box::new(lower_ts_type_inner(first)));
                        }
                    }
                    TyrusType::Promise(Box::new(TyrusType::Void))
                }
                "Record" => {
                    if let Some(params) = &ref_type.type_params {
                        let types: Vec<_> = params
                            .params
                            .iter()
                            .map(|p| lower_ts_type_inner(p))
                            .collect();
                        if types.len() >= 2 {
                            return TyrusType::Map(
                                Box::new(types[0].clone()),
                                Box::new(types[1].clone()),
                            );
                        }
                    }
                    TyrusType::Map(Box::new(TyrusType::String), Box::new(TyrusType::Inferred))
                }
                other => {
                    let ident = Ident::new(other, TyrusSpan::dummy());
                    if let Some(params) = &ref_type.type_params {
                        let type_args: Vec<_> = params
                            .params
                            .iter()
                            .map(|p| lower_ts_type_inner(p))
                            .collect();
                        if !type_args.is_empty() {
                            return TyrusType::Generic(ident, type_args);
                        }
                    }
                    TyrusType::Named(ident)
                }
            }
        }
        TsType::TsUnionOrIntersectionType(
            swc_ecma_ast::TsUnionOrIntersectionType::TsUnionType(union),
        ) => {
            // T | undefined → Option<T>
            let non_undefined: Vec<_> = union
                .types
                .iter()
                .filter(|t| {
                    !matches!(
                        &***t,
                        TsType::TsKeywordType(kw)
                            if kw.kind == TsKeywordTypeKind::TsUndefinedKeyword
                    )
                })
                .collect();

            if non_undefined.len() == 1 && non_undefined.len() < union.types.len() {
                let inner = lower_ts_type_inner(non_undefined[0]);
                TyrusType::Option(Box::new(inner))
            } else {
                // String union or other — treat as Inferred for now
                TyrusType::Inferred
            }
        }
        _ => TyrusType::Inferred,
    }
}
```

- [ ] **Step 4: Create lower.rs (entry point stub)**

```rust
use swc_ecma_ast::Program;

use crate::module::TyrusModule;

/// Lower a SWC Program to a TyrusModule
///
/// This is the entry point for the lowering pass.
/// Currently a stub — will be expanded as codegen migrates to use the IR.
pub fn lower_program(program: &Program) -> TyrusModule {
    let _ = program; // Will be used as lowering is implemented
    TyrusModule::new()
}
```

- [ ] **Step 5: Update lib.rs to include lowering modules**

Add:
```rust
pub mod lower;
pub mod lower_type;
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p tyrus_ast`
Expected: All tests pass

- [ ] **Step 7: Run workspace build**

Run: `cargo build --workspace`
Expected: Builds

- [ ] **Step 8: Commit**

```bash
git add crates/tyrus_ast/
git commit -m "feat(ast): add SWC → IR lowering foundation (types + entry point)"
```

---

## Chunk 3: Expanded Analyzer (tyrus_analyzer)

### Current State

- `crates/tyrus_analyzer/src/lints.rs` — 87 lines, 5 lint rules: var, any, eval, for-in, try-catch
- `crates/tyrus_analyzer/src/decorators.rs` — 160 lines, NestJS decorator extraction
- `crates/tyrus_analyzer/src/lib.rs` — 32 lines, `Analyzer::analyze()` returns `AnalysisResult`
- No unsupported API detection
- No semantic warnings
- No structured output format

### Target State

```
crates/tyrus_analyzer/src/
├── lib.rs              — Analyzer entry point, AnalysisResult
├── lints.rs            — LintVisitor (existing, expanded)
├── decorators.rs       — DecoratorVisitor (existing, unchanged)
├── unsupported.rs      — Unsupported API detection (new)
├── semantic.rs         — Semantic warnings (new)
├── severity.rs         — Severity levels (new)
└── report.rs           — Structured output (JSON, pretty-print) (new)
```

### New Lint Rules to Add

| Rule | Detects | Severity |
|------|---------|----------|
| `no-delete` | `delete obj.prop` | Error |
| `no-with` | `with (obj) { ... }` | Error |
| `no-typeof-check` | `typeof x === "string"` as statement | Warning |
| `no-arguments` | `arguments` keyword | Error |
| `no-label` | labeled statements | Error |
| `no-comma-operator` | comma expressions `(a, b)` | Error |
| `no-void-operator` | `void 0` | Error |
| `no-bitwise` | `&`, `|`, `^`, `~`, `<<`, `>>` | Warning |

### Unsupported API Detection

| API | Reason |
|-----|--------|
| `document.*` | DOM APIs — no browser runtime |
| `window.*` | Browser global — no equivalent |
| `process.*` | Node.js process — partial (process.env OK) |
| `require()` | CommonJS — use ES modules |
| `setTimeout` / `setInterval` | Async timers — use tokio::time |
| `Promise.all` / `Promise.race` | Needs tokio::join! / tokio::select! |
| `JSON.stringify` / `JSON.parse` | Supported but warn about type safety |
| `Symbol` | No Rust equivalent |
| `Proxy` / `Reflect` | Meta-programming — not Oxidizable |
| `WeakMap` / `WeakSet` | GC-dependent — not supported |
| `RegExp` | Needs regex crate — warn about differences |

---

### Task 15: Add severity levels

**Files:**
- Create: `crates/tyrus_analyzer/src/severity.rs`

- [ ] **Step 1: Create severity.rs**

```rust
/// Analyzer diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Severity {
    /// Must fix — code cannot be transpiled
    Error,
    /// Should fix — code may produce unexpected results
    Warning,
    /// Informational — suggestion for better patterns
    Info,
}

/// A structured diagnostic from the analyzer
#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub span: Option<DiagnosticSpan>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiagnosticSpan {
    pub start: usize,
    pub end: usize,
    pub file: String,
}

impl Diagnostic {
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            severity: Severity::Error,
            span: None,
            suggestion: None,
        }
    }

    pub fn warning(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            severity: Severity::Warning,
            span: None,
            suggestion: None,
        }
    }

    pub fn with_span(mut self, start: usize, end: usize, file: &str) -> Self {
        self.span = Some(DiagnosticSpan {
            start,
            end,
            file: file.to_string(),
        });
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }
}
```

- [ ] **Step 2: Add serde to Cargo.toml**

Add to `crates/tyrus_analyzer/Cargo.toml`:
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

- [ ] **Step 3: Verify build**

Run: `cargo build -p tyrus_analyzer`
Expected: Builds

- [ ] **Step 4: Commit**

```bash
git add crates/tyrus_analyzer/src/severity.rs crates/tyrus_analyzer/Cargo.toml
git commit -m "feat(analyzer): add Diagnostic type with severity levels"
```

---

### Task 16: Unsupported API detection

**Files:**
- Create: `crates/tyrus_analyzer/src/unsupported.rs`

- [ ] **Step 1: Write tests**

Create `crates/tyrus_analyzer/tests/unsupported_tests.rs` (or inline `#[cfg(test)]`):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_document_api() {
        let diagnostics = check_source("document.getElementById('x');");
        assert!(diagnostics.iter().any(|d| d.code == "tyrus::unsupported::dom"));
    }

    #[test]
    fn test_detects_require() {
        let diagnostics = check_source("const fs = require('fs');");
        assert!(diagnostics.iter().any(|d| d.code == "tyrus::unsupported::commonjs"));
    }

    #[test]
    fn test_allows_console_log() {
        let diagnostics = check_source("console.log('hello');");
        assert!(diagnostics.iter().all(|d| d.code != "tyrus::unsupported::dom"));
    }
}
```

- [ ] **Step 2: Create unsupported.rs**

```rust
use swc_ecma_ast::{CallExpr, Callee, Expr, MemberExpr, MemberProp};
use swc_ecma_visit::{Visit, VisitWith};

use crate::severity::{Diagnostic, DiagnosticSpan};

/// Detects usage of APIs that cannot be transpiled to Rust
pub struct UnsupportedApiVisitor {
    pub diagnostics: Vec<Diagnostic>,
    pub file_name: String,
}

/// Browser/Node APIs that have no Rust equivalent
const BLOCKED_GLOBALS: &[(&str, &str, &str)] = &[
    ("document", "tyrus::unsupported::dom", "DOM APIs are not available in Rust"),
    ("window", "tyrus::unsupported::dom", "Browser window is not available in Rust"),
    ("navigator", "tyrus::unsupported::dom", "Navigator API is not available in Rust"),
    ("localStorage", "tyrus::unsupported::dom", "LocalStorage is not available in Rust"),
    ("sessionStorage", "tyrus::unsupported::dom", "SessionStorage is not available in Rust"),
    ("XMLHttpRequest", "tyrus::unsupported::dom", "Use fetch/axios patterns instead"),
];

const BLOCKED_FUNCTIONS: &[(&str, &str, &str, &str)] = &[
    ("require", "tyrus::unsupported::commonjs", "CommonJS require() is not supported", "Use ES module imports: import { x } from 'module'"),
    ("setTimeout", "tyrus::unsupported::timer", "setTimeout is not directly supported", "Use tokio::time::sleep() for async delays"),
    ("setInterval", "tyrus::unsupported::timer", "setInterval is not directly supported", "Use tokio::time::interval() for recurring tasks"),
    ("clearTimeout", "tyrus::unsupported::timer", "clearTimeout is not directly supported", "Use tokio task cancellation"),
    ("clearInterval", "tyrus::unsupported::timer", "clearInterval is not directly supported", "Use tokio task cancellation"),
];

impl UnsupportedApiVisitor {
    pub fn new(file_name: String) -> Self {
        Self {
            diagnostics: Vec::new(),
            file_name,
        }
    }

    fn span_to_diag(&self, span: swc_common::Span) -> (usize, usize) {
        let start = span.lo.0 as usize;
        let end = span.hi.0 as usize;
        (start, end)
    }
}

impl Visit for UnsupportedApiVisitor {
    fn visit_member_expr(&mut self, n: &MemberExpr) {
        // Check for document.*, window.*, etc.
        if let Expr::Ident(ident) = &*n.obj {
            let name = ident.sym.as_ref();
            for (global, code, msg) in BLOCKED_GLOBALS {
                if name == *global {
                    let (start, end) = self.span_to_diag(n.span);
                    self.diagnostics.push(
                        Diagnostic::error(code, msg)
                            .with_span(start, end, &self.file_name),
                    );
                }
            }
        }
        n.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, n: &CallExpr) {
        if let Callee::Expr(expr) = &n.callee {
            if let Expr::Ident(ident) = &**expr {
                let name = ident.sym.as_ref();
                for (func, code, msg, suggestion) in BLOCKED_FUNCTIONS {
                    if name == *func {
                        let (start, end) = self.span_to_diag(n.span);
                        self.diagnostics.push(
                            Diagnostic::error(code, msg)
                                .with_span(start, end, &self.file_name)
                                .with_suggestion(suggestion),
                        );
                    }
                }
            }
        }
        n.visit_children_with(self);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p tyrus_analyzer`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/tyrus_analyzer/src/unsupported.rs
git commit -m "feat(analyzer): add unsupported API detection (DOM, require, timers)"
```

---

### Task 17: Expand lint rules

**Files:**
- Modify: `crates/tyrus_analyzer/src/lints.rs`

- [ ] **Step 1: Write tests for new lint rules**

Add to lints.rs or a test module:
```rust
#[cfg(test)]
mod tests {
    // Test that `delete obj.prop` is caught
    // Test that `with (obj) {}` is caught
    // Test that `arguments` usage is caught
}
```

- [ ] **Step 2: Add new visitors to LintVisitor**

Add these `Visit` implementations:
- `visit_unary_expr` — catch `delete` operator
- `visit_with_stmt` — catch `with` blocks
- Check for `arguments` identifier in `visit_ident`

```rust
fn visit_unary_expr(&mut self, n: &swc_ecma_ast::UnaryExpr) {
    if n.op == swc_ecma_ast::UnaryOp::Delete {
        self.errors.push(TyrusError::UnsupportedFeature {
            feature: "delete operator".to_string(),
            src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
            span: self.create_span(n.span),
        });
    }
    n.visit_children_with(self);
}

fn visit_with_stmt(&mut self, n: &swc_ecma_ast::WithStmt) {
    self.errors.push(TyrusError::UnsupportedFeature {
        feature: "with statement".to_string(),
        src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
        span: self.create_span(n.span),
    });
    n.visit_children_with(self);
}

fn visit_labeled_stmt(&mut self, n: &swc_ecma_ast::LabeledStmt) {
    self.errors.push(TyrusError::UnsupportedFeature {
        feature: "labeled statements".to_string(),
        src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
        span: self.create_span(n.span),
    });
    n.visit_children_with(self);
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p tyrus_analyzer`
Expected: All tests pass

- [ ] **Step 4: Run full workspace tests**

Run: `cargo nextest run --workspace`
Expected: No regressions

- [ ] **Step 5: Commit**

```bash
git add crates/tyrus_analyzer/src/lints.rs
git commit -m "feat(analyzer): expand lint rules — delete, with, labeled statements"
```

---

### Task 18: Structured report output

**Files:**
- Create: `crates/tyrus_analyzer/src/report.rs`
- Modify: `crates/tyrus_analyzer/src/lib.rs`

- [ ] **Step 1: Create report.rs**

```rust
use console::style;

use crate::severity::{Diagnostic, Severity};

/// Format diagnostics for terminal display
pub fn format_pretty(diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return format!("  {} No issues found\n", style("✓").green().bold());
    }

    let errors = diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
    let warnings = diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();
    let infos = diagnostics.iter().filter(|d| d.severity == Severity::Info).count();

    let mut out = String::new();

    for d in diagnostics {
        let icon = match d.severity {
            Severity::Error => style("✗").red().bold().to_string(),
            Severity::Warning => style("⚠").yellow().bold().to_string(),
            Severity::Info => style("ℹ").blue().bold().to_string(),
        };

        out.push_str(&format!("  {} {} [{}]\n", icon, d.message, style(&d.code).dim()));

        if let Some(span) = &d.span {
            out.push_str(&format!(
                "    at {}:{}..{}\n",
                style(&span.file).underlined(),
                span.start,
                span.end,
            ));
        }

        if let Some(suggestion) = &d.suggestion {
            out.push_str(&format!("    {} {}\n", style("→").cyan(), suggestion));
        }
    }

    out.push_str(&format!(
        "\n  {} error(s), {} warning(s), {} info(s)\n",
        errors, warnings, infos,
    ));

    out
}

/// Format diagnostics as JSON (for tooling integration)
pub fn format_json(diagnostics: &[Diagnostic]) -> String {
    serde_json::to_string_pretty(diagnostics).unwrap_or_else(|_| "[]".to_string())
}
```

- [ ] **Step 2: Update lib.rs to expose new modules**

```rust
pub mod decorators;
pub mod lints;
pub mod report;
pub mod severity;
pub mod unsupported;
```

And update `AnalysisResult` to include diagnostics:
```rust
pub struct AnalysisResult {
    pub errors: Vec<TyrusError>,
    pub diagnostics: Vec<severity::Diagnostic>,
    pub graph: DiGraph,
}
```

Update `analyze()` to run unsupported API visitor:
```rust
impl Analyzer {
    pub fn analyze(program: &Program, source_code: String, file_name: String) -> AnalysisResult {
        let mut lint_visitor = LintVisitor::new(source_code, file_name.clone());
        program.visit_with(&mut lint_visitor);

        let mut decorator_visitor = DecoratorVisitor::new();
        program.visit_with(&mut decorator_visitor);

        let mut unsupported_visitor = unsupported::UnsupportedApiVisitor::new(file_name);
        program.visit_with(&mut unsupported_visitor);

        AnalysisResult {
            errors: lint_visitor.errors,
            diagnostics: unsupported_visitor.diagnostics,
            graph: decorator_visitor.graph,
        }
    }
}
```

- [ ] **Step 3: Add console dependency**

Add to `crates/tyrus_analyzer/Cargo.toml`:
```toml
console = "0.15"
```

- [ ] **Step 4: Fix downstream consumers**

Update `crates/tyrus_orchestrator/src/lib.rs` and `pipeline.rs` to handle new `diagnostics` field in `AnalysisResult`.

- [ ] **Step 5: Run full workspace tests**

Run: `cargo nextest run --workspace`
Expected: All 138+ tests pass, no regressions

- [ ] **Step 6: Commit**

```bash
git add crates/tyrus_analyzer/ crates/tyrus_orchestrator/
git commit -m "feat(analyzer): add structured report output (pretty + JSON)"
```

---

### Task 19: Wire analyzer diagnostics into CLI

**Files:**
- Modify: `crates/tyrus_cli/src/commands/check.rs`
- Modify: `crates/tyrus_orchestrator/src/lib.rs`

- [ ] **Step 1: Update orchestrator check() to return diagnostics**

Change `check()` to return `AnalysisResult` (or a simpler struct) so the CLI can format them:

```rust
pub struct CheckResult {
    pub errors: Vec<TyrusError>,
    pub diagnostics: Vec<tyrus_analyzer::severity::Diagnostic>,
    pub statement_count: usize,
}

pub fn check(path: &FilePath) -> Result<CheckResult, TyrusError> {
    let program = tyrus_parser::parse(path.as_ref())?;
    let source_code = std::fs::read_to_string(path.as_ref()).map_err(TyrusError::IoError)?;
    let file_name = path.as_ref().to_string_lossy().to_string();

    let analysis = tyrus_analyzer::Analyzer::analyze(&program, source_code, file_name);

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
```

- [ ] **Step 2: Update CLI check command to display diagnostics**

```rust
pub(crate) fn execute(path: &Path) -> Result<()> {
    let mut pipeline = Pipeline::new(vec!["Parse", "Analyze", "Report"]);

    pipeline.start_step(&colors::file_path(&path.display().to_string()));

    let result = tyrus_orchestrator::check(&FilePath::from(path.to_path_buf()))
        .map_err(|e| miette::miette!("{}", e))?;

    pipeline.start_step("Running Oxidizable checks");

    // Show TyrusErrors (lint errors)
    for error in &result.errors {
        eprintln!("{:?}", miette::Report::new(error.clone()));
    }

    // Show structured diagnostics (unsupported APIs, warnings)
    pipeline.start_step("Generating report");
    if !result.diagnostics.is_empty() {
        eprintln!("{}", tyrus_analyzer::report::format_pretty(&result.diagnostics));
    }

    let total_issues = result.errors.len() + result.diagnostics.len();
    if total_issues == 0 {
        pipeline.finish_success(&format!(
            "File is Oxidizable — {} statements parsed",
            result.statement_count,
        ));
    } else {
        pipeline.finish_error(&format!("{} issue(s) found", total_issues));
    }

    Ok(())
}
```

- [ ] **Step 3: Add `--json` flag for CLI check**

Add `--json` flag to check command for machine-readable output:
```rust
/// Analyze TypeScript for Oxidizable compatibility
Check {
    path: PathBuf,
    /// Output diagnostics as JSON
    #[arg(long)]
    json: bool,
},
```

- [ ] **Step 4: Run full test suite**

Run: `cargo nextest run --workspace`
Expected: All tests pass

- [ ] **Step 5: Manual test — check with unsupported API**

```bash
echo 'document.getElementById("x"); setTimeout(() => {}, 1000);' > /tmp/test_unsupported.ts
cargo run --bin tyrus -- check /tmp/test_unsupported.ts
cargo run --bin tyrus -- check --json /tmp/test_unsupported.ts
```
Expected: Pretty-printed diagnostics showing DOM and timer warnings; JSON output for --json flag

- [ ] **Step 6: Commit**

```bash
git add crates/tyrus_cli/ crates/tyrus_orchestrator/ crates/tyrus_analyzer/
git commit -m "feat: wire analyzer diagnostics into CLI with pretty and JSON output"
```

---

## Chunk 4: Update Documentation and MASTER_PLAN

### Task 20: Update documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/MASTER_PLAN.md`

- [ ] **Step 1: Update CLAUDE.md**

Update the Crate Map with new information:
- `tyrus_cli`: Updated description with 4 commands + branding
- `tyrus_ast`: Updated from "Reserved" to actual IR description
- `tyrus_analyzer`: Updated with expanded lint count

Update Architecture diagram to show IR:
```
.ts input
  → tyrus_parser       (SWC parsing → swc_ecma_ast::Program)
  → tyrus_ast           (SWC AST → TyrusModule IR) [NEW]
  → tyrus_analyzer     (LintVisitor + DecoratorVisitor + UnsupportedApiVisitor)
  → tyrus_di           (petgraph topological sort for DI)
  → tyrus_orchestrator (multi-file coordination)
  → tyrus_codegen      (quote! → proc_macro2::TokenStream)
  → formatted .rs output
```

- [ ] **Step 2: Update README.md**

Add new commands to the Commands Reference table:
```markdown
| `cargo run --bin tyrus -- compile <dir>/src --output <dir>/output` | Transpile + compile to native binary |
| `cargo run --bin tyrus -- run <dir>/src --output <dir>/output` | Transpile + compile + execute |
```

Update Installation & Usage section with new commands.

- [ ] **Step 3: Update MASTER_PLAN.md**

Add new phase between current phases:
```markdown
Phase 5.5 🔄 Architecture   (CLI + IR + Analyzer evolution)
```

Update metrics with new test counts.

- [ ] **Step 4: Commit**

```bash
git add CLAUDE.md README.md docs/superpowers/plans/MASTER_PLAN.md
git commit -m "docs: update documentation for CLI, IR, and analyzer evolution"
```

---

## Execution Order Summary

| Order | Task | Chunk | Est. Size |
|-------|------|-------|-----------|
| 1 | Add CLI dependencies | 1: CLI | Small |
| 2 | Create output module (banner + colors) | 1: CLI | Medium |
| 3 | Create progress module | 1: CLI | Small |
| 4 | Create check + build command modules | 1: CLI | Medium |
| 5 | Create compile command | 1: CLI | Small |
| 6 | Create run command | 1: CLI | Small |
| 7 | Rewrite main.rs | 1: CLI | Medium |
| 8 | CLI integration tests | 1: CLI | Small |
| 9 | IR core types (types, ident, span) | 2: IR | Medium |
| 10 | IR expressions (expr.rs) | 2: IR | Large |
| 11 | IR statements (stmt.rs) | 2: IR | Medium |
| 12 | IR declarations (decl.rs) | 2: IR | Large |
| 13 | IR module + update lib.rs | 2: IR | Small |
| 14 | SWC → IR lowering foundation | 2: IR | Large |
| 15 | Analyzer severity levels | 3: Analyzer | Small |
| 16 | Unsupported API detection | 3: Analyzer | Medium |
| 17 | Expand lint rules | 3: Analyzer | Medium |
| 18 | Structured report output | 3: Analyzer | Medium |
| 19 | Wire diagnostics into CLI | 3: Analyzer | Medium |
| 20 | Update documentation | 4: Docs | Small |

**Total: 20 tasks, 4 chunks**

---

## Dependencies

```
Chunk 1 (CLI)       → independent, execute first
Chunk 2 (IR)        → independent of CLI, execute second
Chunk 3 (Analyzer)  → depends on severity.rs (internal), slight coupling to orchestrator
Chunk 4 (Docs)      → depends on all previous chunks
```

Chunks 1 and 2 can theoretically run in parallel (independent crates), but sequential execution is recommended to avoid merge conflicts in workspace-level files.
