//! Arrow function expression code generation.
//!
//! Converts TypeScript arrow functions to Rust closures.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr, Pat};

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

        // Save and reset state flag — closures don't capture `state` from handler scope
        let saved_flag = self.use_state_for_this.get();
        self.use_state_for_this.set(false);

        let body = match &*arrow.body {
            BlockStmtOrExpr::BlockStmt(block) => {
                let stmts: Vec<_> = block.stmts.iter().map(|s| self.convert_stmt(s)).collect();
                quote! { { #(#stmts)* } }
            }
            BlockStmtOrExpr::Expr(expr) => self.convert_expr(expr),
        };

        self.use_state_for_this.set(saved_flag);

        quote! {
            |#(#params),*| #body
        }
    }
}
