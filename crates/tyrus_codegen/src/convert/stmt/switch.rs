//! Switch statement conversion (switch → match).

use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::Stmt;

use super::super::interface::RustGenerator;

impl RustGenerator {
    /// Convert a switch statement to a Rust match expression.
    /// Each case with a body becomes a match arm. Fall-through is not supported.
    pub(crate) fn convert_switch_stmt(&self, switch: &swc_ecma_ast::SwitchStmt) -> TokenStream {
        let discriminant = self.convert_expr(&switch.discriminant);
        let mut arms = Vec::new();

        for case in &switch.cases {
            let body: Vec<_> = case
                .cons
                .iter()
                .filter(|s| !matches!(s, Stmt::Break(_)))
                .map(|s| self.convert_stmt(s))
                .collect();

            if body.is_empty() {
                continue;
            }

            if let Some(test) = &case.test {
                let test_expr = self.convert_expr(test);
                arms.push(quote! { __v if __v == #test_expr => { #(#body)* } });
            } else {
                arms.push(quote! { _ => { #(#body)* } });
            }
        }

        let has_default = switch.cases.iter().any(|c| c.test.is_none());
        if !has_default {
            arms.push(quote! { _ => {} });
        }

        quote! {
            match #discriminant {
                #(#arms)*
            }
        }
    }
}
