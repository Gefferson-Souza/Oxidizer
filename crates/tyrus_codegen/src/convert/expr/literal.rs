//! Literal expression code generation.
//!
//! Handles: number/string/bool/null literals, object literals (→ serde_json::json!),
//! array literals (→ vec![]), and template literals (→ format!).

use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use crate::convert::interface::RustGenerator;

impl RustGenerator {
    pub(crate) fn convert_lit(&self, lit: &Lit) -> TokenStream {
        match lit {
            Lit::Num(num) => {
                let value = num.value;
                quote! { #value }
            }
            Lit::Str(s) => {
                let v = s.value.as_str().unwrap_or("");
                quote! { String::from(#v) }
            }
            Lit::Bool(b) => {
                let v = b.value;
                quote! { #v }
            }
            Lit::Null(_) => quote! { None },
            _ => quote! { compile_error!("Tyrus: unsupported literal type") },
        }
    }

    pub(crate) fn convert_object_lit(&self, obj: &swc_ecma_ast::ObjectLit) -> TokenStream {
        let mut fields = Vec::new();
        for prop in &obj.props {
            if let swc_ecma_ast::PropOrSpread::Prop(p) = prop {
                if let swc_ecma_ast::Prop::KeyValue(kv) = &**p {
                    if let swc_ecma_ast::PropName::Ident(ident) = &kv.key {
                        let key = ident.sym.as_ref();

                        let is_null = matches!(&*kv.value, Expr::Lit(Lit::Null(_)));
                        let is_undefined = if let Expr::Ident(id) = &*kv.value {
                            id.sym.as_ref() == "undefined"
                        } else {
                            false
                        };

                        if is_null || is_undefined {
                            fields.push(quote! { #key: serde_json::Value::Null });
                        } else {
                            let val = self.convert_expr(&kv.value);
                            fields.push(quote! { #key: #val });
                        }
                    }
                }
            }
        }
        quote! { serde_json::json!({ #(#fields),* }) }
    }

    pub(crate) fn convert_array_lit(&self, arr: &swc_ecma_ast::ArrayLit) -> TokenStream {
        let elems: Vec<_> = arr
            .elems
            .iter()
            .flatten()
            .map(|elem| self.convert_expr_or_spread(elem))
            .collect();
        quote! { vec![#(#elems),*] }
    }

    pub(crate) fn convert_tpl(&self, tpl: &swc_ecma_ast::Tpl) -> TokenStream {
        let mut fmt_str = String::new();
        let mut args = Vec::new();

        for (i, quasi) in tpl.quasis.iter().enumerate() {
            if let Some(cooked) = &quasi.cooked {
                fmt_str.push_str(cooked.as_str().unwrap_or(quasi.raw.as_str()));
            } else {
                fmt_str.push_str(quasi.raw.as_str());
            }

            if i < tpl.exprs.len() {
                fmt_str.push_str("{}");
                args.push(self.convert_expr(&tpl.exprs[i]));
            }
        }

        quote! { format!(#fmt_str, #(#args),*) }
    }
}
