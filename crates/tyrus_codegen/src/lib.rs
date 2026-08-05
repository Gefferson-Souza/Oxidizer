#![forbid(unsafe_code)]

pub mod convert;
pub(crate) mod decorators;
pub mod stdlib;

use convert::interface::RustGenerator;
use proc_macro2::TokenStream;
use std::str::FromStr;
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
        generator.code.push('\n');
        generator
            .code
            .push_str(&wrap_top_level_in_main(&generator.main_body));
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

/// Wraps top-level script statements in `fn main() { ... }`.
///
/// Per Power of Ten Rule 7, code emission goes through `quote!` rather
/// than string concatenation. Statements arrive already converted to
/// Rust syntax (via `convert_stmt`), so we parse them as a TokenStream
/// and embed under a function header. Parse failure surfaces via
/// `compile_error!` rather than silent body drop.
fn wrap_top_level_in_main(body: &str) -> String {
    let body_tokens = TokenStream::from_str(body).unwrap_or_else(|_| {
        quote::quote! {
            compile_error!("Tyrus: internal error — top-level statement TokenStream parse failed");
        }
    });
    let wrapped = quote::quote! {
        fn main() {
            #body_tokens
        }
    };
    format!("{wrapped}\n")
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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::wrap_top_level_in_main;

    #[test]
    fn wrap_emits_fn_main_for_valid_body() {
        let out = wrap_top_level_in_main("println!(\"hi\");");
        assert!(out.contains("fn main"), "expected fn main: {out}");
        assert!(out.contains("println"), "expected println in body: {out}");
    }

    #[test]
    fn wrap_fallback_emits_compile_error_on_unbalanced_tokens() {
        // Token-level parse failure (unpaired delimiters) hits the
        // unwrap_or_else fallback and must surface via compile_error!.
        let out = wrap_top_level_in_main("fn { invalid {{{{");
        assert!(
            out.contains("compile_error"),
            "fallback should emit compile_error!, got: {out}"
        );
        assert!(
            out.contains("Tyrus:"),
            "fallback message should carry Tyrus prefix: {out}"
        );
    }

    #[test]
    fn wrap_emits_empty_main_for_empty_body() {
        let out = wrap_top_level_in_main("");
        assert!(
            out.contains("fn main"),
            "even empty body still gets fn main wrapper: {out}"
        );
    }
}
