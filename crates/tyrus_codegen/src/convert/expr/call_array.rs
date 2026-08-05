//! Array (and a couple of string) method-call code generation.
//!
//! Extracted from `convert/expr/call.rs` to keep that file under the Rule 4
//! 400-line ceiling (`POWER_OF_TEN.md`). The dispatch entry point
//! [`RustGenerator::try_convert_array_method`] is invoked from
//! `convert/expr/call.rs::convert_general_call`. All handlers live in this
//! module because they share the same dispatch + helper surface
//! (`convert_call_args`, `has_index_callback_arg`, `try_inline_filter`).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{CallExpr, Expr, MemberExpr, Pat};

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    /// Dispatches array/string method calls to specialized handlers.
    pub(crate) fn try_convert_array_method(
        &self,
        member: &MemberExpr,
        call: &CallExpr,
    ) -> Option<TokenStream> {
        let method_ident = member.prop.as_ident()?;
        let method_name = method_ident.sym.as_ref();
        let obj = self.convert_expr(&member.obj);

        // Owned here: array methods needing IR-aware emission (TS callback
        // closures, state-field detection in push, replacen overload). Pure
        // Vec-op methods (find, join, slice, includes, indexOf, concat,
        // reverse, pop, sort, shift, flat, flatMap) live in
        // `crate::stdlib::array`. See ADR 0012 for the ownership boundary.
        match method_name {
            "map" => self.convert_map_call(&obj, call),
            "filter" => self.convert_filter_call(&obj, call),
            "forEach" => self.convert_for_each_call(&obj, call),
            "some" => Some(self.convert_iter_adaptor_call(&obj, call, "any")),
            "every" => Some(self.convert_iter_adaptor_call(&obj, call, "all")),
            "reduce" => Some(self.convert_reduce_call(&obj, call)),
            "push" => Some(self.convert_push_call(member, call)),
            "replace" => Some(self.convert_replace_call(member, call)),
            _ => None,
        }
    }

    /// `arr.map(callback)` -- with optional index parameter support.
    fn convert_map_call(&self, obj: &TokenStream, call: &CallExpr) -> Option<TokenStream> {
        let args = self.convert_call_args(call);

        if Self::has_index_callback_arg(call) {
            let closure = args.first()?;
            return Some(quote! {
                #obj.iter().cloned()
                    .enumerate()
                    .map(|(i, v)| (#closure)(v, i as f64))
                    .collect::<Vec<_>>()
            });
        }

        Some(quote! { #obj.iter().cloned().map(#(#args),*).collect::<Vec<_>>() })
    }

    /// `arr.filter(callback)` -- inline arrow or closure wrapper.
    fn convert_filter_call(&self, obj: &TokenStream, call: &CallExpr) -> Option<TokenStream> {
        let args = self.convert_call_args(call);

        if Self::has_index_callback_arg(call) {
            let closure = args.first()?;
            return Some(quote! {
                #obj.iter().cloned()
                    .enumerate()
                    .filter(|(i, v)| (#closure)(v.clone(), *i as f64))
                    .map(|(_, v)| v)
                    .collect::<Vec<_>>()
            });
        }

        if let Some(tokens) = self.try_inline_filter(obj, call) {
            return Some(tokens);
        }

        let closure = args.first()?;
        Some(quote! {
            #obj.iter().cloned()
                .filter(|__v| (#closure)(__v.clone()))
                .collect::<Vec<_>>()
        })
    }

    /// `arr.forEach(callback)` -- with optional index parameter support.
    fn convert_for_each_call(&self, obj: &TokenStream, call: &CallExpr) -> Option<TokenStream> {
        let args = self.convert_call_args(call);

        if Self::has_index_callback_arg(call) {
            let closure = args.first()?;
            return Some(quote! {
                #obj.iter().cloned()
                    .enumerate()
                    .for_each(|(i, v)| (#closure)(v, i as f64))
            });
        }

        Some(quote! { #obj.iter().cloned().for_each(#(#args),*) })
    }

    /// Shared handler for `some` -> `any`, `every` -> `all`, `find` -> `find`.
    fn convert_iter_adaptor_call(
        &self,
        obj: &TokenStream,
        call: &CallExpr,
        rust_method: &str,
    ) -> TokenStream {
        let args = self.convert_call_args(call);
        let method = format_ident!("{}", rust_method);
        quote! { #obj.iter().cloned().#method(#(#args),*) }
    }

    /// `arr.reduce(callback, initial?)` -> `fold` or `reduce`.
    fn convert_reduce_call(&self, obj: &TokenStream, call: &CallExpr) -> TokenStream {
        let closure = call.args.first().map_or_else(
            || {
                quote! { compile_error!("Tyrus: reduce requires a callback") }
            },
            |a| self.convert_expr(&a.expr),
        );

        if let Some(second) = call.args.get(1) {
            let initial = self.convert_expr(&second.expr);
            quote! { #obj.iter().cloned().fold(#initial, #closure) }
        } else {
            quote! { #obj.iter().cloned().reduce(#closure) }
        }
    }

    /// Converts all call arguments to `TokenStream` values.
    fn convert_call_args(&self, call: &CallExpr) -> Vec<TokenStream> {
        call.args
            .iter()
            .map(|a| self.convert_expr(&a.expr))
            .collect()
    }

    fn has_index_callback_arg(call: &CallExpr) -> bool {
        if let Some(arg) = call.args.first() {
            if let Expr::Arrow(arrow) = &*arg.expr {
                return arrow.params.len() == 2;
            }
            if let Expr::Fn(fn_expr) = &*arg.expr {
                return fn_expr.function.params.len() == 2;
            }
        }
        false
    }

    fn try_inline_filter(&self, obj: &TokenStream, call: &CallExpr) -> Option<TokenStream> {
        let first_arg = call.args.first()?;
        if let Expr::Arrow(arrow) = &*first_arg.expr {
            if let Some(Pat::Ident(param_ident)) = arrow.params.first() {
                let param_name = format_ident!("{}", to_snake_case(param_ident.sym.as_ref()));
                let body = match &*arrow.body {
                    swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => self.convert_expr(expr),
                    swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => {
                        let stmts: Vec<_> =
                            block.stmts.iter().map(|s| self.convert_stmt(s)).collect();
                        quote! { #(#stmts)* }
                    }
                };
                return Some(quote! {
                    #obj.clone().into_iter()
                        .filter(|#param_name| { let #param_name = #param_name.clone(); #body })
                        .collect::<Vec<_>>()
                });
            }
        }
        None
    }

    fn convert_push_call(&self, member: &MemberExpr, call: &CallExpr) -> TokenStream {
        let method_ident = format_ident!("push");
        let args: Vec<_> = call
            .args
            .iter()
            .map(|a| self.convert_expr(&a.expr))
            .collect();

        if let Expr::Member(nested_member) = &*member.obj {
            if nested_member.obj.is_this() {
                if let Some(prop_ident) = nested_member.prop.as_ident() {
                    let prop_name = to_snake_case(prop_ident.sym.as_ref());
                    let field = format_ident!("{}", prop_name);
                    // If it's a state field wrapped in Arc<Mutex<>>, lock before push
                    // Clone args to avoid move since caller may use them after push
                    if self
                        .current_class_state_fields
                        .contains_key(prop_ident.sym.as_ref())
                    {
                        let cloned_args: Vec<_> =
                            args.iter().map(|a| quote! { #a.clone() }).collect();
                        return quote! { self.#field.lock().unwrap_or_else(|e| e.into_inner()).#method_ident(#(#cloned_args),*) };
                    }
                    return quote! { self.#field.#method_ident(#(#args),*) };
                }
            }
        }

        if member.obj.is_this() {
            return quote! { self.#method_ident(#(#args),*) };
        }

        let obj = self.convert_expr(&member.obj);
        quote! { #obj.#method_ident(#(#args),*) }
    }

    fn convert_replace_call(&self, member: &MemberExpr, call: &CallExpr) -> TokenStream {
        let args: Vec<_> = call
            .args
            .iter()
            .map(|a| self.convert_expr(&a.expr))
            .collect();
        let obj = self.convert_expr(&member.obj);

        if args.len() == 2 {
            let method_ident = format_ident!("replacen");
            quote! { #obj.#method_ident(#(#args),*, 1) }
        } else {
            quote! { #obj.replace(#(#args),*) }
        }
    }
}
