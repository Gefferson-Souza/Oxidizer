//! Statement-level code generation.
//!
//! Converts TypeScript statements (variable declarations, if/else, loops,
//! return, throw, switch) to Rust TokenStreams.

mod loops;
mod switch;
mod var_decl;

use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::interface::RustGenerator;

impl RustGenerator {
    /// Recursively convert statements with a custom return handler.
    /// Used by process_fn_decl to inject Result wrapping for async functions.
    pub fn convert_stmt_recursive<F>(&self, stmt: &Stmt, return_handler: &mut F) -> TokenStream
    where
        F: FnMut(&swc_ecma_ast::ReturnStmt) -> TokenStream,
    {
        match stmt {
            Stmt::Return(ret_stmt) => return_handler(ret_stmt),
            Stmt::Block(block) => {
                let stmts: Vec<_> = block
                    .stmts
                    .iter()
                    .map(|s| self.convert_stmt_recursive(s, return_handler))
                    .collect();
                quote! { { #(#stmts)* } }
            }
            Stmt::If(if_stmt) => {
                let test = self.convert_expr(&if_stmt.test);
                let cons = self.convert_stmt_recursive(&if_stmt.cons, return_handler);
                let cons_block = if matches!(*if_stmt.cons, Stmt::Block(_)) {
                    quote! { #cons }
                } else {
                    quote! { { #cons } }
                };

                let alt = if let Some(alt) = &if_stmt.alt {
                    let alt_stmt = self.convert_stmt_recursive(alt, return_handler);
                    let alt_block = if matches!(&**alt, Stmt::Block(_) | Stmt::If(_)) {
                        quote! { #alt_stmt }
                    } else {
                        quote! { { #alt_stmt } }
                    };
                    quote! { else #alt_block }
                } else {
                    quote! {}
                };

                quote! { if #test #cons_block #alt }
            }
            _ => self.convert_stmt(stmt),
        }
    }

    /// Convert a single TypeScript statement to Rust tokens.
    pub fn convert_stmt(&self, stmt: &Stmt) -> TokenStream {
        match stmt {
            Stmt::Return(ret_stmt) => {
                if let Some(arg) = &ret_stmt.arg {
                    let expr = self.convert_expr(arg);
                    quote! { return #expr; }
                } else {
                    quote! { return; }
                }
            }
            Stmt::Expr(expr_stmt) => {
                let expr = self.convert_expr(&expr_stmt.expr);
                quote! { #expr; }
            }
            Stmt::Decl(Decl::Var(var_decl)) => self.convert_var_decl(var_decl),
            Stmt::Block(block) => {
                let stmts: Vec<_> = block.stmts.iter().map(|s| self.convert_stmt(s)).collect();
                quote! { { #(#stmts)* } }
            }
            Stmt::If(if_stmt) => {
                let test = self.convert_expr(&if_stmt.test);
                let cons = self.convert_stmt(&if_stmt.cons);
                let cons_block = if matches!(*if_stmt.cons, Stmt::Block(_)) {
                    quote! { #cons }
                } else {
                    quote! { { #cons } }
                };

                let alt = if let Some(alt) = &if_stmt.alt {
                    let alt_stmt = self.convert_stmt(alt);
                    let alt_block = if matches!(&**alt, Stmt::Block(_) | Stmt::If(_)) {
                        quote! { #alt_stmt }
                    } else {
                        quote! { { #alt_stmt } }
                    };
                    quote! { else #alt_block }
                } else {
                    quote! {}
                };

                quote! { if #test #cons_block #alt }
            }
            Stmt::While(while_stmt) => self.convert_while_stmt(while_stmt),
            Stmt::ForOf(for_of) => self.convert_for_of_stmt(for_of),
            Stmt::ForIn(for_in) => self.convert_for_in_stmt(for_in),
            Stmt::For(for_stmt) => self.convert_for_stmt(for_stmt),
            Stmt::DoWhile(do_while) => self.convert_do_while_stmt(do_while),
            Stmt::Switch(switch_stmt) => self.convert_switch_stmt(switch_stmt),
            Stmt::Throw(throw_stmt) => {
                let arg = self.convert_expr(&throw_stmt.arg);
                quote! { return Err(#arg.into()); }
            }
            _ => quote! { /* other stmts todo */ },
        }
    }
}
