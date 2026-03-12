//! Expression-level code generation.
//!
//! Dispatches TypeScript expressions to specialized handlers in submodules.

mod arrow;
mod binary;
mod call;
mod literal;
mod member;
mod misc;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use super::helpers::to_snake_case;
use super::interface::RustGenerator;

impl RustGenerator {
    /// Main expression dispatcher — routes each expression type to its handler.
    pub fn convert_expr(&self, expr: &Expr) -> TokenStream {
        match expr {
            Expr::Bin(bin) => self.convert_bin_expr(bin),
            Expr::This(_) => quote! { self },
            Expr::Ident(ident) => {
                let name = ident.sym.as_str();
                if name == "undefined" {
                    return quote! { None };
                }
                if name.chars().next().is_some_and(char::is_uppercase) {
                    let ident_token = format_ident!("{}", name);
                    quote! { #ident_token }
                } else {
                    let ident_name = to_snake_case(name);
                    let ident_token = format_ident!("{}", ident_name);
                    quote! { #ident_token }
                }
            }
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
            Expr::Paren(paren) => self.convert_expr(&paren.expr),
            Expr::Arrow(arrow) => self.convert_arrow_expr(arrow),
            Expr::Array(arr) => self.convert_array_lit(arr),
            Expr::Tpl(tpl) => self.convert_tpl(tpl),
            Expr::Cond(cond) => {
                let test = self.convert_expr(&cond.test);
                let cons = self.convert_expr(&cond.cons);
                let alt = self.convert_expr(&cond.alt);
                quote! { if #test { #cons } else { #alt } }
            }
            Expr::OptChain(opt_chain) => self.convert_opt_chain(opt_chain),
            _ => quote! { compile_error!("Tyrus: unsupported expression") },
        }
    }

    pub fn convert_expr_or_spread(&self, arg: &ExprOrSpread) -> TokenStream {
        self.convert_expr(&arg.expr)
    }

    fn convert_new_expr(&self, new_expr: &NewExpr) -> TokenStream {
        let callee = self.convert_expr(&new_expr.callee);
        let args: Vec<TokenStream> = new_expr
            .args
            .as_ref()
            .map(|a| a.iter().map(|arg| self.convert_expr(&arg.expr)).collect())
            .unwrap_or_default();
        quote! { #callee::new(#(#args),*) }
    }
}
