//! Miscellaneous expression code generation.
//!
//! Handles: assignment expressions, update expressions (++/--),
//! and optional chaining (?.).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    pub(crate) fn convert_assign_expr(&self, assign: &swc_ecma_ast::AssignExpr) -> TokenStream {
        let right = self.convert_expr(&assign.right);
        let left = match &assign.left {
            swc_ecma_ast::AssignTarget::Simple(simple) => match simple {
                swc_ecma_ast::SimpleAssignTarget::Member(member) => {
                    if member.obj.is_this() {
                        if let Some(prop_ident) = member.prop.as_ident() {
                            let prop_name = to_snake_case(prop_ident.sym.as_ref());
                            let field = format_ident!("{}", prop_name);

                            if self
                                .current_class_state_fields
                                .contains_key(prop_ident.sym.as_ref())
                            {
                                quote! { *self.#field.lock().unwrap_or_else(|e| e.into_inner()) }
                            } else {
                                quote! { self.#field }
                            }
                        } else {
                            quote! { compile_error!("Tyrus: unsupported self member assignment") }
                        }
                    } else if let Some(prop_ident) = member.prop.as_ident() {
                        let prop_name = to_snake_case(prop_ident.sym.as_ref());
                        let field = format_ident!("{}", prop_name);
                        let obj = self.convert_expr(&member.obj);
                        quote! { #obj.#field }
                    } else {
                        quote! { compile_error!("Tyrus: unsupported member assignment pattern") }
                    }
                }
                swc_ecma_ast::SimpleAssignTarget::Ident(ident) => {
                    let name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                    quote! { #name }
                }
                _ => quote! { compile_error!("Tyrus: unsupported assignment target") },
            },
            _ => quote! { compile_error!("Tyrus: unsupported assignment pattern") },
        };

        match assign.op {
            swc_ecma_ast::AssignOp::Assign => quote! { #left = #right },
            swc_ecma_ast::AssignOp::AddAssign => quote! { #left += #right },
            _ => quote! { #left op= #right },
        }
    }

    pub(crate) fn convert_update_expr(&self, update: &UpdateExpr) -> TokenStream {
        let arg = self.convert_expr(&update.arg);
        match update.op {
            UpdateOp::PlusPlus => quote! { #arg += 1.0 },
            UpdateOp::MinusMinus => quote! { #arg -= 1.0 },
        }
    }

    pub(crate) fn convert_unary_expr(&self, expr: &UnaryExpr) -> TokenStream {
        let arg = self.convert_expr(&expr.arg);
        match expr.op {
            UnaryOp::Minus => quote! { -(#arg) },
            UnaryOp::Plus => arg,
            UnaryOp::Bang => quote! { !(#arg) },
            UnaryOp::TypeOf => {
                quote! { compile_error!("Tyrus: typeof not supported") }
            }
            _ => quote! { compile_error!("Tyrus: unsupported unary operator") },
        }
    }

    pub(crate) fn convert_opt_chain(&self, opt_chain: &swc_ecma_ast::OptChainExpr) -> TokenStream {
        match &*opt_chain.base {
            swc_ecma_ast::OptChainBase::Member(member) => {
                let obj = self.convert_expr(&member.obj);
                if let Some(prop_ident) = member.prop.as_ident() {
                    let prop_name = format_ident!("{}", to_snake_case(prop_ident.sym.as_ref()));
                    quote! { #obj.as_ref().and_then(|__v| Some(__v.#prop_name.clone())) }
                } else {
                    quote! { #obj }
                }
            }
            swc_ecma_ast::OptChainBase::Call(call) => {
                let callee = self.convert_expr(&call.callee);
                let args: Vec<_> = call
                    .args
                    .iter()
                    .map(|a| self.convert_expr(&a.expr))
                    .collect();
                quote! { #callee(#(#args),*) }
            }
        }
    }
}
