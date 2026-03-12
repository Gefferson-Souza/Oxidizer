//! Member expression code generation.
//!
//! Handles property access: `obj.prop`, `this.field`, enum access (`Status::Active`),
//! computed properties (`arr[0]`), and Mutex-wrapped state fields.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    pub(crate) fn convert_member_expr(&self, member: &MemberExpr) -> TokenStream {
        if member.obj.is_this() {
            if let Some(prop_ident) = member.prop.as_ident() {
                let prop_name = to_snake_case(prop_ident.sym.as_ref());
                let field_ident = format_ident!("{}", prop_name);

                // CHECK STATE FIELDS
                if let Some(type_str) = self.current_class_state_fields.get(prop_ident.sym.as_ref())
                {
                    let needs_deref = matches!(
                        type_str.as_str(),
                        "f64" | "bool" | "i32" | "usize" | "u64" | "i64"
                    );

                    if needs_deref {
                        return quote! { *self.#field_ident.lock().unwrap_or_else(|e| e.into_inner()) };
                    } else if type_str == "String" {
                        return quote! { self.#field_ident.lock().unwrap_or_else(|e| e.into_inner()).clone() };
                    } else {
                        // Collection/complex types: clone to release MutexGuard
                        return quote! { self.#field_ident.lock().unwrap_or_else(|e| e.into_inner()).clone() };
                    }
                } else {
                    return quote! { self.#field_ident.clone() };
                }
            }
        }

        let obj = self.convert_expr(&member.obj);
        match &member.prop {
            swc_ecma_ast::MemberProp::Ident(ident) => {
                let name = ident.sym.as_ref();

                // Map TS .length to Rust .len() (cast to f64 for TS number compat)
                if name == "length" {
                    return quote! { #obj.len() as f64 };
                }

                let prop_name = if name.chars().next().is_some_and(char::is_uppercase) {
                    name.to_string()
                } else {
                    to_snake_case(name)
                };
                let prop = format_ident!("{}", prop_name);

                // Heuristic: if obj is a single Capitalized word, treat as Enum/Static access
                let obj_str = obj.to_string();
                if obj_str.chars().next().is_some_and(char::is_uppercase)
                    && !obj_str.contains('.')
                    && !obj_str.contains('(')
                {
                    return quote! { #obj::#prop };
                }

                quote! { #obj.#prop }
            }
            swc_ecma_ast::MemberProp::Computed(computed) => {
                // Unwrap parens
                let mut expr = &*computed.expr;
                while let Expr::Paren(p) = expr {
                    expr = &p.expr;
                }

                if let Expr::Lit(Lit::Num(n)) = expr {
                    let idx = n.value as usize;
                    quote! { #obj[#idx].clone() }
                } else {
                    let expr_tokens = self.convert_expr(&computed.expr);
                    quote! { #obj[#expr_tokens].clone() }
                }
            }
            _ => quote! { compile_error!("Tyrus: unsupported member expression") },
        }
    }
}
