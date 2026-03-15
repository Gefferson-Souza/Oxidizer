//! Getter and setter transpilation.
//!
//! Converts TypeScript `get prop()` → `fn prop(&self) -> T`
//! and `set prop(v)` → `fn set_prop(&mut self, v: T)`.

use quote::{format_ident, quote};
use swc_ecma_ast::Pat;

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;
use crate::convert::type_mapper::map_ts_type;

impl RustGenerator {
    /// Converts a getter method (`get prop()`) into `fn prop(&self) -> T`.
    pub(crate) fn convert_getter(
        &self,
        method: &swc_ecma_ast::ClassMethod,
    ) -> proc_macro2::TokenStream {
        let prop_name = if let Some(ident) = method.key.as_ident() {
            to_snake_case(ident.sym.as_ref())
        } else {
            return quote! { /* unsupported getter key */ };
        };
        let getter_ident = format_ident!("{}", prop_name);
        let return_type = map_ts_type(method.function.return_type.as_ref());

        let body_stmts = self.convert_getter_body(method);

        quote! {
            pub fn #getter_ident(&self) -> #return_type {
                #(#body_stmts)*
            }
        }
    }

    /// Converts a setter method (`set prop(v)`) into `fn set_prop(&mut self, v: T)`.
    pub(crate) fn convert_setter(
        &self,
        method: &swc_ecma_ast::ClassMethod,
    ) -> proc_macro2::TokenStream {
        let prop_name = if let Some(ident) = method.key.as_ident() {
            to_snake_case(ident.sym.as_ref())
        } else {
            return quote! { /* unsupported setter key */ };
        };
        let setter_ident = format_ident!("set_{}", prop_name);

        let mut params = Vec::new();
        for param in &method.function.params {
            if let Pat::Ident(ident) = &param.pat {
                let param_name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                let param_type = map_ts_type(ident.type_ann.as_ref());
                params.push(quote! { #param_name: #param_type });
            }
        }

        let body_stmts = self.convert_setter_body(method);

        quote! {
            pub fn #setter_ident(&mut self, #(#params),*) {
                #(#body_stmts)*
            }
        }
    }

    /// Builds getter body statements.
    fn convert_getter_body(
        &self,
        method: &swc_ecma_ast::ClassMethod,
    ) -> Vec<proc_macro2::TokenStream> {
        let Some(body) = &method.function.body else {
            return Vec::new();
        };
        body.stmts.iter().map(|s| self.convert_stmt(s)).collect()
    }

    /// Builds setter body statements.
    fn convert_setter_body(
        &self,
        method: &swc_ecma_ast::ClassMethod,
    ) -> Vec<proc_macro2::TokenStream> {
        let Some(body) = &method.function.body else {
            return Vec::new();
        };
        body.stmts.iter().map(|s| self.convert_stmt(s)).collect()
    }
}
