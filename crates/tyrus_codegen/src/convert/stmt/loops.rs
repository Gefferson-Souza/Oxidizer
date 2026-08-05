//! Loop statement conversion (while, for-of, for-in, for, do-while).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{Decl, DoWhileStmt, ForInStmt, ForOfStmt, ForStmt, Pat, Stmt, WhileStmt};

use super::super::helpers::to_snake_case;
use super::super::interface::RustGenerator;

/// Extract loop variable ident from a `ForHead` declaration.
fn extract_loop_var(left: &swc_ecma_ast::ForHead, fallback: &str) -> proc_macro2::Ident {
    match left {
        swc_ecma_ast::ForHead::VarDecl(var_decl) => {
            if let Some(decl) = var_decl.decls.first() {
                if let Pat::Ident(ident) = &decl.name {
                    return format_ident!("{}", to_snake_case(&ident.id.sym));
                }
            }
            format_ident!("{}", fallback)
        }
        swc_ecma_ast::ForHead::Pat(pat) => {
            if let Pat::Ident(ident) = pat.as_ref() {
                format_ident!("{}", to_snake_case(&ident.id.sym))
            } else {
                format_ident!("{}", fallback)
            }
        }
        swc_ecma_ast::ForHead::UsingDecl(_) => format_ident!("{}", fallback),
    }
}

/// Wrap statement output in a block if it isn't already one.
fn ensure_block(stmt: &Stmt, tokens: TokenStream) -> TokenStream {
    if matches!(stmt, Stmt::Block(_)) {
        tokens
    } else {
        quote! { { #tokens } }
    }
}

impl RustGenerator {
    // Bound: SWC AST is finite; recursion depth ≤ AST depth (Rule 1, POWER_OF_TEN.md).
    pub(crate) fn convert_while_stmt(&self, while_stmt: &WhileStmt) -> TokenStream {
        let test = self.convert_expr(&while_stmt.test);
        let body = self.convert_stmt(&while_stmt.body);
        let body_block = ensure_block(&while_stmt.body, body);
        quote! { while #test #body_block }
    }

    // Bound: SWC AST is finite; recursion depth ≤ AST depth (Rule 1, POWER_OF_TEN.md).
    pub(crate) fn convert_for_of_stmt(&self, for_of: &ForOfStmt) -> TokenStream {
        let body = self.convert_stmt(&for_of.body);
        let right = self.convert_expr(&for_of.right);
        let body_block = ensure_block(&for_of.body, body);
        let var_ident = extract_loop_var(&for_of.left, "_item");
        quote! { for #var_ident in #right #body_block }
    }

    // Bound: SWC AST is finite; recursion depth ≤ AST depth (Rule 1, POWER_OF_TEN.md).
    pub(crate) fn convert_for_in_stmt(&self, for_in: &ForInStmt) -> TokenStream {
        let body = self.convert_stmt(&for_in.body);
        let right = self.convert_expr(&for_in.right);
        let body_block = ensure_block(&for_in.body, body);
        let var_ident = extract_loop_var(&for_in.left, "_key");
        quote! { for #var_ident in #right.keys() #body_block }
    }

    // Bound: SWC AST is finite; recursion depth ≤ AST depth (Rule 1, POWER_OF_TEN.md).
    pub(crate) fn convert_for_stmt(&self, for_stmt: &ForStmt) -> TokenStream {
        let init = match &for_stmt.init {
            Some(swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl)) => {
                self.convert_stmt(&Stmt::Decl(Decl::Var(var_decl.clone())))
            }
            Some(swc_ecma_ast::VarDeclOrExpr::Expr(expr)) => {
                let e = self.convert_expr(expr);
                quote! { #e; }
            }
            None => quote! {},
        };
        let test = for_stmt
            .test
            .as_ref()
            .map_or_else(|| quote! { true }, |t| self.convert_expr(t));
        let update = for_stmt
            .update
            .as_ref()
            .map(|u| {
                let expr = self.convert_expr(u);
                quote! { #expr; }
            })
            .unwrap_or_default();
        let body = self.convert_stmt(&for_stmt.body);
        let body_block = if matches!(*for_stmt.body, Stmt::Block(_)) {
            body
        } else {
            quote! { { #body } }
        };

        // for(init; test; update) { body }
        // -> { init; while test { body; update; } }
        quote! {
            {
                #init
                while #test {
                    #body_block
                    #update
                }
            }
        }
    }

    // Bound: SWC AST is finite; recursion depth ≤ AST depth (Rule 1, POWER_OF_TEN.md).
    pub(crate) fn convert_do_while_stmt(&self, do_while: &DoWhileStmt) -> TokenStream {
        let test = self.convert_expr(&do_while.test);
        let body = self.convert_stmt(&do_while.body);
        let body_block = if matches!(*do_while.body, Stmt::Block(_)) {
            body
        } else {
            quote! { { #body } }
        };

        // do { body } while (cond) -> loop { body; if !cond { break; } }
        quote! {
            loop {
                #body_block
                if !(#test) { break; }
            }
        }
    }
}
