pub mod convert;
pub(crate) mod decorators;
pub mod stdlib;

use convert::interface::RustGenerator;
use swc_ecma_ast::Program;
use swc_ecma_visit::VisitWith;

#[derive(Debug, Clone)]
pub struct ControllerMetadata {
    pub struct_name: String,
    pub route_path: String,
}

pub struct GeneratedCode {
    pub code: String,
    pub controllers: Vec<ControllerMetadata>,
}

pub fn generate(program: &Program, is_index: bool) -> GeneratedCode {
    let mut generator = RustGenerator::new(is_index);
    program.visit_with(&mut generator);

    if !generator.main_body.is_empty() && !generator.has_declared_main {
        // Wrap top-level statements in fn main() only if no main function already declared.
        generator.code.push_str("\nfn main() {\n");
        generator.code.push_str(&generator.main_body);
        generator.code.push_str("}\n");
    }

    let code = if generator.needs_int_serde.get() {
        format!("{}\n{}", INT_SERDE_HELPER_MODULE, generator.code)
    } else {
        generator.code
    };

    GeneratedCode {
        code,
        controllers: generator.controllers,
    }
}

/// Module emitted at the top of any generated file that uses the
/// integer-shape serde attribute (see `convert::integer_heuristic`).
/// File-local so the path resolves under `assert_rust_compiles`
/// (standalone snippet) and inside multi-module projects alike.
const INT_SERDE_HELPER_MODULE: &str = r#"
mod __tyrus_int_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(value: &f64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_i64(*value as i64)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        v.as_f64().ok_or_else(|| serde::de::Error::custom("expected a number"))
    }
}
"#;
