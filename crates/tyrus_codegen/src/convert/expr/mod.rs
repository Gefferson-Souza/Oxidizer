//! Expression-level code generation.
//!
//! Dispatches TypeScript expressions to specialized handlers in submodules.

mod arrow;
mod binary;
mod call;
mod literal;
mod member;
mod misc;

fn convert_ident(ident: &swc_ecma_ast::Ident) -> TokenStream {
    let name = ident.sym.as_str();
    if name == "undefined" {
        return quote! { None };
    }
    if name.chars().next().is_some_and(char::is_uppercase) {
        let ident_token = format_ident!("{}", name);
        quote! { #ident_token }
    } else {
        let ident_name = super::helpers::to_snake_case(name);
        let ident_token = format_ident!("{}", ident_name);
        quote! { #ident_token }
    }
}

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use super::interface::RustGenerator;

impl RustGenerator {
    /// Main expression dispatcher — routes each expression type to its handler.
    pub fn convert_expr(&self, expr: &Expr) -> TokenStream {
        match expr {
            Expr::Bin(bin) => self.convert_bin_expr(bin),
            Expr::This(_) => {
                if self.use_state_for_this.get() {
                    quote! { state }
                } else {
                    quote! { self }
                }
            }
            Expr::Ident(ident) => convert_ident(ident),
            Expr::Lit(lit) => self.convert_lit(lit),
            Expr::Member(member) => self.convert_member_expr(member),
            Expr::Call(call) => self.convert_call_expr(call),
            Expr::Object(obj) => self.convert_object_lit(obj),
            Expr::Assign(assign) => self.convert_assign_expr(assign),
            Expr::Update(update) => self.convert_update_expr(update),
            Expr::Await(await_expr) => {
                let arg = self.convert_expr(&await_expr.arg);
                quote! { #arg.await? }
            }
            Expr::New(new_expr) => self.convert_new_expr(new_expr),
            Expr::Paren(paren) => {
                let inner = self.convert_expr(&paren.expr);
                quote! { (#inner) }
            }
            Expr::Arrow(arrow) => self.convert_arrow_expr(arrow),
            Expr::Array(arr) => self.convert_array_lit(arr),
            Expr::Tpl(tpl) => self.convert_tpl(tpl),
            Expr::Cond(cond) => self.convert_cond_expr(cond),
            Expr::OptChain(opt_chain) => self.convert_opt_chain(opt_chain),
            Expr::Unary(unary) => self.convert_unary_expr(unary),
            // Type assertions: mostly no-ops (types known at compile time)
            Expr::TsAs(ts_as) => self.convert_expr(&ts_as.expr),
            Expr::TsTypeAssertion(ta) => self.convert_expr(&ta.expr),
            Expr::TsConstAssertion(tca) => self.convert_expr(&tca.expr),
            Expr::TsNonNull(nn) => self.convert_expr(&nn.expr),
            _ => quote! { compile_error!("Tyrus: unsupported expression") },
        }
    }

    pub fn convert_expr_or_spread(&self, arg: &ExprOrSpread) -> TokenStream {
        self.convert_expr(&arg.expr)
    }

    fn convert_cond_expr(&self, cond: &swc_ecma_ast::CondExpr) -> TokenStream {
        let test = self.convert_expr(&cond.test);
        let cons = self.convert_expr(&cond.cons);
        let alt = self.convert_expr(&cond.alt);
        quote! { if #test { #cons } else { #alt } }
    }

    fn convert_new_expr(&self, new_expr: &NewExpr) -> TokenStream {
        if let Expr::Ident(ident) = &*new_expr.callee {
            match ident.sym.as_ref() {
                "Map" => return quote! { std::collections::HashMap::new() },
                "Set" => return quote! { std::collections::HashSet::new() },
                _ => {}
            }
        }

        let callee = self.convert_expr(&new_expr.callee);
        let args: Vec<TokenStream> = new_expr
            .args
            .as_ref()
            .map(|a| a.iter().map(|arg| self.convert_expr(&arg.expr)).collect())
            .unwrap_or_default();
        quote! { #callee::new(#(#args),*) }
    }
}
