//! Call expression code generation.
//!
//! Handles: stdlib calls, axios/fetch -> reqwest, top-level dispatch.
//! Array and string method-call handlers live in
//! [`crate::convert::expr::call_array`] (split out for the Rule 4
//! 400-line file ceiling).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{CallExpr, Callee, Expr, Lit, MemberExpr};

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    // Bound: SWC AST is finite; recursion depth ≤ AST depth via convert_expr on
    // callee + args (Rule 1, POWER_OF_TEN.md).
    pub(crate) fn convert_call_expr(&self, call: &CallExpr) -> TokenStream {
        // Try stdlib handlers first
        if let Some(stdlib_code) =
            crate::stdlib::try_handle_stdlib_call(self, &call.callee, &call.args)
        {
            return stdlib_code;
        }

        // Check for member-expression patterns (axios, static, stdlib methods)
        if let Some(tokens) = self.try_convert_member_call(call) {
            return tokens;
        }

        // Handle fetch()
        if let Some(tokens) = self.try_convert_fetch_call(call) {
            return tokens;
        }

        // Handle array/string method calls and general calls
        self.convert_general_call(call)
    }

    /// Handles member-expression calls: static methods, axios, and stdlib methods.
    fn try_convert_member_call(&self, call: &CallExpr) -> Option<TokenStream> {
        let Callee::Expr(expr) = &call.callee else {
            return None;
        };
        let Expr::Member(member) = &**expr else {
            return None;
        };

        if let Some(tokens) = self.try_convert_static_call(member, call) {
            return Some(tokens);
        }
        if let Some(tokens) = self.try_convert_axios_call(member, call) {
            return Some(tokens);
        }
        if let Some(method_ident) = member.prop.as_ident() {
            let method_name = method_ident.sym.as_ref();
            if let Some(stdlib_code) =
                crate::stdlib::try_handle_method_call(self, &member.obj, method_name, &call.args)
            {
                return Some(stdlib_code);
            }
        }
        None
    }

    /// Converts `ClassName.method(args)` to `ClassName::method(args)`.
    fn try_convert_static_call(&self, member: &MemberExpr, call: &CallExpr) -> Option<TokenStream> {
        let Expr::Ident(obj_ident) = &*member.obj else {
            return None;
        };
        let class_name = obj_ident.sym.as_ref();
        let static_set = self.static_methods.get(class_name)?;
        let method_ident = member.prop.as_ident()?;
        let method_str = method_ident.sym.as_ref();

        if !static_set.contains(method_str) {
            return None;
        }

        let cls = format_ident!("{}", class_name);
        let meth = format_ident!("{}", to_snake_case(method_str));
        let args: Vec<_> = call
            .args
            .iter()
            .map(|a| self.convert_expr(&a.expr))
            .collect();
        Some(quote! { #cls::#meth(#(#args),*) })
    }

    /// Fallback: array/string method check, then plain callee + args.
    fn convert_general_call(&self, call: &CallExpr) -> TokenStream {
        let callee = if let Callee::Expr(expr) = &call.callee {
            if let Expr::Member(member) = &**expr {
                if let Some(tokens) = self.try_convert_array_method(member, call) {
                    return tokens;
                }
                // this.method(args) → self.method(args)
                if member.obj.is_this() {
                    if let Some(prop) = member.prop.as_ident() {
                        let method = format_ident!("{}", to_snake_case(prop.sym.as_ref()));
                        let args: Vec<_> = call
                            .args
                            .iter()
                            .map(|a| self.convert_expr(&a.expr))
                            .collect();
                        let receiver = if self.use_state_for_this.get() {
                            quote! { state }
                        } else {
                            quote! { self }
                        };
                        return quote! { #receiver.#method(#(#args),*) };
                    }
                }
            }
            self.convert_expr(expr)
        } else {
            quote! { compile_error!("Tyrus: unsupported call expression") }
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
                            .map_or_else(|| quote! { "" }, |a| self.convert_expr(&a.expr));
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
                        .map_or_else(|| quote! { "" }, |a| self.convert_expr(&a.expr));
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
}
