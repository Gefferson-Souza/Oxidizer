//! Constructor transpilation: TS `constructor(...)` → Rust `new()` + `new_di()`.
//!
//! Split by responsibility:
//!   - [`params`] — extract typed parameters and TS-param-prop auto field inits
//!   - [`field_init`] — walk the body, lift `super(...)` and `this.x = y` into
//!     field initializers, wrap in `Some/Arc/Arc<Mutex>` as needed
//!   - [`di`] — build the parallel `new_di(...)` constructor for the DI graph
//!
//! `convert_constructor` (the dispatcher) lives in this module so the impl on
//! `RustGenerator` keeps its current public-on-the-crate surface.

mod di;
mod field_init;
mod params;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::Constructor;

use crate::convert::interface::RustGenerator;

use di::build_di_constructor;
use field_init::{extract_field_inits, fill_missing_fields, FieldInitCtx};
use params::extract_constructor_params;

/// Context for constructor conversion, replacing many positional arguments.
pub(crate) struct ConstructorCtx<'a> {
    pub(crate) constructor: &'a Constructor,
    pub(crate) class_fields: &'a [(String, bool)],
    pub(crate) has_generics: bool,
    pub(crate) generic_params: &'a HashSet<String>,
    pub(crate) dependency_fields: &'a HashSet<String>,
    pub(crate) is_service_or_controller: bool,
}

/// Check whether a type annotation refers to a dependency (non-primitive TypeRef).
pub(crate) fn is_dependency_type(
    type_ann: Option<&swc_ecma_ast::TsTypeAnn>,
    generic_params: &HashSet<String>,
) -> bool {
    let ann = match type_ann {
        Some(a) => a,
        None => return false,
    };
    let type_ref = match ann.type_ann.as_ts_type_ref() {
        Some(r) => r,
        None => return false,
    };
    match type_ref.type_name.as_ident() {
        Some(ident) => {
            let name = ident.sym.as_str();
            if generic_params.contains(name) {
                return false;
            }
            !matches!(
                name,
                "String" | "f64" | "bool" | "i32" | "Vec" | "Option" | "Array"
            )
        }
        None => true,
    }
}

impl RustGenerator {
    /// Dispatcher: convert a TypeScript constructor into `new()` + `new_di()` methods.
    pub(crate) fn convert_constructor(
        &self,
        _struct_name: &proc_macro2::Ident,
        ctx: &ConstructorCtx<'_>,
    ) -> TokenStream {
        let mut extracted = extract_constructor_params(
            ctx.constructor,
            ctx.generic_params,
            ctx.is_service_or_controller,
        );

        let init_ctx = FieldInitCtx {
            class_fields: ctx.class_fields,
            dependency_fields: ctx.dependency_fields,
            dependency_params: &extracted.dependency_params,
            is_service_or_controller: ctx.is_service_or_controller,
        };

        extract_field_inits(
            self,
            ctx.constructor,
            &init_ctx,
            &mut extracted.field_inits,
            &mut extracted.initialized_fields,
        );

        fill_missing_fields(
            ctx.class_fields,
            ctx.has_generics,
            &extracted.initialized_fields,
            &mut extracted.field_inits,
        );

        let params = &extracted.params;
        let field_inits = &extracted.field_inits;

        if !field_inits.is_empty() {
            let di_tokens = build_di_constructor(ctx.constructor, ctx);
            quote! {
                pub fn new(#(#params),*) -> Self {
                    Self {
                        #(#field_inits),*
                    }
                }

                #di_tokens
            }
        } else {
            quote! {
                pub fn new(#(#params),*) -> Self {
                    compile_error!("Tyrus: complex constructor pattern not yet supported")
                }
            }
        }
    }
}
