//! Try-catch-finally statement conversion.
//!
//! TypeScript try-catch maps to Rust Result matching:
//! ```text
//! try { body } catch (e) { handler }
//! →
//! match (|| -> Result<_, String> { body })() {
//!     Ok(__v) => { return __v; },
//!     Err(e) => { handler }
//! }
//! ```

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{BlockStmt, CatchClause, Expr, IfStmt, Pat, Stmt, TryStmt};

use super::super::interface::RustGenerator;

fn stmt_has_return(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return(_) => true,
        Stmt::If(if_stmt) => {
            stmt_has_return(&if_stmt.cons) || if_stmt.alt.as_deref().is_some_and(stmt_has_return)
        }
        Stmt::Block(block) => block.stmts.iter().any(stmt_has_return),
        Stmt::Try(try_stmt) => try_stmt.block.stmts.iter().any(stmt_has_return),
        _ => false,
    }
}

impl RustGenerator {
    /// Convert a try-catch-finally statement to Rust Result matching.
    pub(crate) fn convert_try_stmt(&self, try_stmt: &TryStmt) -> TokenStream {
        let try_body = self.convert_try_body(&try_stmt.block);
        let catch_arm = self.convert_catch_clause(try_stmt.handler.as_ref());
        let finally_body = self.convert_finally_block(try_stmt.finalizer.as_ref());

        let match_expr = quote! {
            match (|| -> Result<_, String> {
                #try_body
            })() {
                Ok(__try_result) => { return __try_result; },
                #catch_arm
            }
        };

        if finally_body.is_empty() {
            match_expr
        } else {
            quote! {
                {
                    let __finally_result = #match_expr;
                    #finally_body
                    __finally_result
                }
            }
        }
    }

    fn convert_try_body(&self, block: &BlockStmt) -> TokenStream {
        let stmts: Vec<_> = block
            .stmts
            .iter()
            .map(|s| self.convert_try_inner_stmt(s))
            .collect();

        let has_return = block.stmts.iter().any(stmt_has_return);

        if has_return {
            quote! { #(#stmts)* }
        } else {
            quote! { #(#stmts)* Ok(Default::default()) }
        }
    }

    /// Convert a statement inside a try block — wraps `return` in `Ok()`,
    /// `throw` in `Err()`.
    fn convert_try_inner_stmt(&self, stmt: &Stmt) -> TokenStream {
        match stmt {
            Stmt::Return(ret_stmt) => {
                if let Some(arg) = &ret_stmt.arg {
                    let expr = self.convert_expr(arg);
                    quote! { return Ok(#expr); }
                } else {
                    quote! { return Ok(()); }
                }
            }
            Stmt::If(if_stmt) => self.convert_try_if_stmt(if_stmt),
            Stmt::Block(block) => {
                let inner: Vec<_> = block
                    .stmts
                    .iter()
                    .map(|s| self.convert_try_inner_stmt(s))
                    .collect();
                quote! { { #(#inner)* } }
            }
            Stmt::Throw(throw_stmt) => self.convert_throw_in_try(&throw_stmt.arg),
            Stmt::Try(inner_try) => {
                // Nested try-catch: inner catch returns go back into
                // the outer try closure, so wrap catch body returns in Ok()
                let inner_body = self.convert_try_body(&inner_try.block);
                let inner_catch = self.convert_catch_clause_for_nesting(inner_try.handler.as_ref());

                quote! {
                    match (|| -> Result<_, String> {
                        #inner_body
                    })() {
                        Ok(__inner_result) => { return Ok(__inner_result); },
                        #inner_catch
                    }
                }
            }
            _ => self.convert_stmt(stmt),
        }
    }

    fn convert_try_if_stmt(&self, if_stmt: &IfStmt) -> TokenStream {
        let test = self.convert_expr(&if_stmt.test);
        let cons = self.convert_try_inner_stmt(&if_stmt.cons);
        let cons_block = if matches!(*if_stmt.cons, Stmt::Block(_)) {
            cons
        } else {
            quote! { { #cons } }
        };

        let alt = if let Some(alt) = &if_stmt.alt {
            let alt_stmt = self.convert_try_inner_stmt(alt);
            let alt_block = if matches!(&**alt, Stmt::Block(_) | Stmt::If(_)) {
                alt_stmt
            } else {
                quote! { { #alt_stmt } }
            };
            quote! { else #alt_block }
        } else {
            quote! {}
        };

        quote! { if #test #cons_block #alt }
    }

    /// Convert a throw expression inside a try block.
    fn convert_throw_in_try(&self, arg: &Expr) -> TokenStream {
        if let Expr::New(new_expr) = arg {
            if let Expr::Ident(ident) = &*new_expr.callee {
                if ident.sym.as_ref() == "Error" {
                    if let Some(args) = &new_expr.args {
                        if let Some(first_arg) = args.first() {
                            let msg = self.convert_expr(&first_arg.expr);
                            return quote! { return Err(#msg.to_string()); };
                        }
                    }
                    return quote! { return Err(String::from("Error")); };
                }
            }
        }

        let expr = self.convert_expr(arg);
        quote! { return Err(#expr.to_string()); }
    }

    fn convert_catch_clause(&self, handler: Option<&CatchClause>) -> TokenStream {
        let Some(handler) = handler else {
            return quote! { Err(_) => {} };
        };

        let error_ident = if let Some(Pat::Ident(ident)) = &handler.param {
            format_ident!("{}", ident.id.sym.to_string())
        } else {
            format_ident!("_error")
        };

        let body_stmts: Vec<_> = handler
            .body
            .stmts
            .iter()
            .map(|s| self.convert_stmt(s))
            .collect();

        quote! {
            Err(#error_ident) => {
                let #error_ident = #error_ident;
                #(#body_stmts)*
            }
        }
    }

    /// Catch clause for nested try-catch: wraps return values in `Ok()`
    /// so they propagate correctly through the outer try closure.
    fn convert_catch_clause_for_nesting(&self, handler: Option<&CatchClause>) -> TokenStream {
        let Some(handler) = handler else {
            return quote! { Err(_) => {} };
        };

        let error_ident = if let Some(Pat::Ident(ident)) = &handler.param {
            format_ident!("{}", ident.id.sym.to_string())
        } else {
            format_ident!("_error")
        };

        let body_stmts: Vec<_> = handler
            .body
            .stmts
            .iter()
            .map(|s| self.convert_try_inner_stmt(s))
            .collect();

        quote! {
            Err(#error_ident) => {
                let #error_ident = #error_ident;
                #(#body_stmts)*
            }
        }
    }

    fn convert_finally_block(&self, finalizer: Option<&BlockStmt>) -> TokenStream {
        let Some(block) = finalizer else {
            return quote! {};
        };

        let stmts: Vec<_> = block.stmts.iter().map(|s| self.convert_stmt(s)).collect();

        quote! { #(#stmts)* }
    }
}
