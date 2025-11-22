use ox_diagnostics::OxidizerError;
use std::path::PathBuf;

pub fn check(path: PathBuf) -> Result<(), OxidizerError> {
    let program = ox_parser::parse(&path)?;
    let count = match program {
        swc_ecma_ast::Program::Module(m) => m.body.len(),
        swc_ecma_ast::Program::Script(s) => s.body.len(),
    };
    println!("✅ AST parsed successfully with {} statements", count);
    Ok(())
}

pub fn pipeline() -> Result<(), OxidizerError> {
    // Stub implementation
    Ok(())
}
