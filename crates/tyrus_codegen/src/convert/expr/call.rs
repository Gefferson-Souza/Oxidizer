//! Call expression code generation.
//!
//! Handles: stdlib calls, axios/fetch → reqwest, array methods
//! (map, filter, forEach, find, some, every, reduce, push),
//! and string methods (replace → replacen).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    pub(crate) fn convert_call_expr(&self, call: &CallExpr) -> TokenStream {
        // Try stdlib handlers first
        if let Some(stdlib_code) =
            crate::stdlib::try_handle_stdlib_call(self, &call.callee, &call.args)
        {
            return stdlib_code;
        }

        // Check for axios and stdlib method calls
        if let Callee::Expr(expr) = &call.callee {
            if let Expr::Member(member) = &**expr {
                if let Some(tokens) = self.try_convert_axios_call(member, call) {
                    return tokens;
                }
                if let Some(method_ident) = member.prop.as_ident() {
                    let method_name = method_ident.sym.as_ref();
                    if let Some(stdlib_code) = crate::stdlib::try_handle_method_call(
                        self,
                        &member.obj,
                        method_name,
                        &call.args,
                    ) {
                        return stdlib_code;
                    }
                }
            }
        }

        // Handle fetch()
        if let Some(tokens) = self.try_convert_fetch_call(call) {
            return tokens;
        }

        // Handle array/string method calls and general calls
        let callee = match &call.callee {
            Callee::Expr(expr) => {
                if let Expr::Member(member) = &**expr {
                    if let Some(tokens) = self.try_convert_array_method(member, call) {
                        return tokens;
                    }
                }
                self.convert_expr(expr)
            }
            _ => quote! { compile_error!("Tyrus: unsupported call expression") },
        };
        let args: Vec<_> = call
            .args
            .iter()
            .map(|a| self.convert_expr(&a.expr))
            .collect();
        quote! { #callee(#(#args),*) }
    }

    fn try_convert_axios_call(&self, member: &MemberExpr, call: &CallExpr) -> Option<TokenStream> {
        if let Expr::Ident(obj_ident) = &*member.obj {
            if obj_ident.sym.as_str() == "axios" {
                if let Some(method_ident) = member.prop.as_ident() {
                    let method_name = method_ident.sym.as_str();
                    if matches!(method_name, "get" | "post" | "put" | "delete" | "patch") {
                        let url = call
                            .args
                            .first()
                            .map(|a| self.convert_expr(&a.expr))
                            .unwrap_or_else(|| quote! { "" });
                        let method_fn = format_ident!("{}", method_name);
                        return Some(quote! { reqwest::Client::new().#method_fn(#url).send() });
                    }
                }
            }
        }
        None
    }

    fn try_convert_fetch_call(&self, call: &CallExpr) -> Option<TokenStream> {
        if let Callee::Expr(expr) = &call.callee {
            if let Expr::Ident(ident) = &**expr {
                if ident.sym.as_str() == "fetch" {
                    let url = call
                        .args
                        .first()
                        .map(|a| self.convert_expr(&a.expr))
                        .unwrap_or_else(|| quote! { "" });
                    let method = call.args.get(1).and_then(|opts| {
                        if let Expr::Object(obj) = &*opts.expr {
                            for prop in &obj.props {
                                if let swc_ecma_ast::PropOrSpread::Prop(p) = prop {
                                    if let swc_ecma_ast::Prop::KeyValue(kv) = &**p {
                                        if let swc_ecma_ast::PropName::Ident(id) = &kv.key {
                                            if id.sym.as_str() == "method" {
                                                if let Expr::Lit(Lit::Str(s)) = &*kv.value {
                                                    return Some(
                                                        s.value
                                                            .as_str()
                                                            .unwrap_or_default()
                                                            .to_uppercase(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        None
                    });
                    let method_call = match method.as_deref() {
                        Some("POST") => quote! { post },
                        Some("PUT") => quote! { put },
                        Some("DELETE") => quote! { delete },
                        Some("PATCH") => quote! { patch },
                        _ => quote! { get },
                    };
                    return Some(quote! { reqwest::Client::new().#method_call(#url).send() });
                }
            }
        }
        None
    }

    #[allow(clippy::too_many_lines)]
    fn try_convert_array_method(
        &self,
        member: &MemberExpr,
        call: &CallExpr,
    ) -> Option<TokenStream> {
        let method_ident = member.prop.as_ident()?;
        let method_name = method_ident.sym.as_ref();

        match method_name {
            "map" | "filter" => {
                let obj = self.convert_expr(&member.obj);
                let rust_method = format_ident!("{}", to_snake_case(method_name));
                let args: Vec<_> = call
                    .args
                    .iter()
                    .map(|a| self.convert_expr(&a.expr))
                    .collect();

                let has_index_arg = self.has_index_callback_arg(call);

                if has_index_arg {
                    let closure = &args[0];
                    if method_name == "filter" {
                        return Some(quote! {
                            #obj.clone().into_iter()
                                .enumerate()
                                .filter(|(i, v)| (#closure)(v.clone(), *i as f64))
                                .map(|(_, v)| v)
                                .collect::<Vec<_>>()
                        });
                    } else {
                        return Some(quote! {
                            #obj.clone().into_iter()
                                .enumerate()
                                .map(|(i, v)| (#closure)(v, i as f64))
                                .collect::<Vec<_>>()
                        });
                    }
                }

                if method_name == "filter" {
                    if let Some(tokens) = self.try_inline_filter(&obj, call) {
                        return Some(tokens);
                    }
                    let closure = &args[0];
                    return Some(quote! {
                        #obj.clone().into_iter()
                            .filter(|__v| (#closure)(__v.clone()))
                            .collect::<Vec<_>>()
                    });
                }
                Some(
                    quote! { #obj.clone().into_iter().#rust_method(#(#args),*).collect::<Vec<_>>() },
                )
            }
            "forEach" => {
                let obj = self.convert_expr(&member.obj);
                let args: Vec<_> = call
                    .args
                    .iter()
                    .map(|a| self.convert_expr(&a.expr))
                    .collect();

                if self.has_index_callback_arg(call) {
                    let closure = &args[0];
                    return Some(quote! {
                        #obj.into_iter()
                            .enumerate()
                            .for_each(|(i, v)| (#closure)(v, i as f64))
                    });
                }
                Some(quote! { #obj.into_iter().for_each(#(#args),*) })
            }
            "some" => {
                let obj = self.convert_expr(&member.obj);
                let args: Vec<_> = call
                    .args
                    .iter()
                    .map(|a| self.convert_expr(&a.expr))
                    .collect();
                Some(quote! { #obj.into_iter().any(#(#args),*) })
            }
            "every" => {
                let obj = self.convert_expr(&member.obj);
                let args: Vec<_> = call
                    .args
                    .iter()
                    .map(|a| self.convert_expr(&a.expr))
                    .collect();
                Some(quote! { #obj.into_iter().all(#(#args),*) })
            }
            "find" | "reduce" => {
                let obj = self.convert_expr(&member.obj);
                let rust_method = format_ident!("{}", to_snake_case(method_name));
                let args: Vec<_> = call
                    .args
                    .iter()
                    .map(|a| self.convert_expr(&a.expr))
                    .collect();
                Some(quote! { #obj.into_iter().#rust_method(#(#args),*) })
            }
            "push" => Some(self.convert_push_call(member, call)),
            "replace" => Some(self.convert_replace_call(member, call)),
            _ => None,
        }
    }

    fn has_index_callback_arg(&self, call: &CallExpr) -> bool {
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
