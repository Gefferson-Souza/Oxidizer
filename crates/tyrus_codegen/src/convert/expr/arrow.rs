//! Arrow function expression code generation.
//!
//! Converts TypeScript arrow functions to Rust closures.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    pub(crate) fn convert_arrow_expr(&self, arrow: &ArrowExpr) -> TokenStream {
        let params: Vec<_> = arrow
            .params
            .iter()
            .map(|pat| {
                if let Pat::Ident(ident) = pat {
                    let name = format_ident!("{}", to_snake_case(&ident.id.sym));
                    quote! { #name }
                } else {
                    quote! { _ }
                }
            })
            .collect();

        let body = match &*arrow.body {
            BlockStmtOrExpr::BlockStmt(block) => {
                let stmts: Vec<_> = block.stmts.iter().map(|s| self.convert_stmt(s)).collect();
                quote! { { #(#stmts)* } }
            }
            BlockStmtOrExpr::Expr(expr) => self.convert_expr(expr),
        };

        quote! {
            |#(#params),*| #body
        }
    }
}
