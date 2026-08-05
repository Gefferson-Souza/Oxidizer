//! Literal expression code generation.
//!
//! Handles: number/string/bool/null literals, object literals (→ serde_json::json!),
//! array literals (→ vec![]), and template literals (→ format!).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use crate::convert::helpers::to_snake_case;
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
                match &**p {
                    swc_ecma_ast::Prop::KeyValue(kv) => {
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
                    swc_ecma_ast::Prop::Shorthand(ident) => {
                        let key = ident.sym.as_ref();
                        let val_name = super::convert_ident(ident);
                        fields.push(quote! { #key: #val_name });
                    }
                    _ => {}
                }
            }
        }
        quote! { serde_json::json!({ #(#fields),* }) }
    }

    pub(crate) fn convert_array_lit(&self, arr: &swc_ecma_ast::ArrayLit) -> TokenStream {
        let elems: Vec<_> = arr.elems.iter().flatten().collect::<Vec<_>>();

        // Check if any element uses spread syntax
        let has_spread = elems.iter().any(|e| e.spread.is_some());

        if !has_spread {
            let items: Vec<_> = elems.iter().map(|e| self.convert_expr(&e.expr)).collect();
            return quote! { vec![#(#items),*] };
        }

        // Build chain of iterators for spread elements
        let mut chains: Vec<TokenStream> = Vec::new();
        let mut pending_items: Vec<TokenStream> = Vec::new();

        for elem in &elems {
            if elem.spread.is_some() {
                // Flush pending non-spread items first
                if !pending_items.is_empty() {
                    let items = std::mem::take(&mut pending_items);
                    chains.push(quote! { vec![#(#items),*].into_iter() });
                }
                let spread_expr = self.convert_expr(&elem.expr);
                chains.push(quote! { #spread_expr.iter().cloned() });
            } else {
                pending_items.push(self.convert_expr(&elem.expr));
            }
        }

        // Flush remaining non-spread items
        if !pending_items.is_empty() {
            let items = pending_items;
            chains.push(quote! { vec![#(#items),*].into_iter() });
        }

        // Chain all iterators together
        if chains.len() == 1 {
            let first = &chains[0];
            quote! { #first.collect::<Vec<_>>() }
        } else {
            let first = &chains[0];
            let rest = &chains[1..];
            quote! { #first #(.chain(#rest))* .collect::<Vec<_>>() }
        }
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

    /// Try to convert an object literal with a known type annotation into a struct constructor.
    /// e.g., `const x: User = { id: 1, name: "foo" }` → `User { id: 1f64, name: String::from("foo") }`
    /// Returns None if type annotation can't be extracted (falls back to serde_json::json!).
    pub(crate) fn try_convert_typed_object_lit(
        &self,
        ts_type: &swc_ecma_ast::TsType,
        obj: &swc_ecma_ast::ObjectLit,
    ) -> Option<TokenStream> {
        let swc_ecma_ast::TsType::TsTypeRef(type_ref) = ts_type else {
            return None;
        };
        let type_name = type_ref.type_name.as_ident()?.sym.to_string();

        let type_ident = format_ident!("{}", type_name);
        let mut fields = Vec::new();

        let mut spread_base: Option<TokenStream> = None;

        for prop in &obj.props {
            match prop {
                swc_ecma_ast::PropOrSpread::Spread(spread) => {
                    spread_base = Some(self.convert_expr(&spread.expr));
                }
                swc_ecma_ast::PropOrSpread::Prop(p) => {
                    if let swc_ecma_ast::Prop::KeyValue(kv) = &**p {
                        if let swc_ecma_ast::PropName::Ident(key_ident) = &kv.key {
                            let field_name =
                                format_ident!("{}", to_snake_case(key_ident.sym.as_ref()));
                            let val = self.convert_expr(&kv.value);
                            fields.push(quote! { #field_name: #val });
                        }
                    } else if let swc_ecma_ast::Prop::Shorthand(shorthand) = &**p {
                        let field_name = format_ident!("{}", to_snake_case(shorthand.sym.as_ref()));
                        fields.push(quote! { #field_name: #field_name.clone() });
                    }
                }
            }
        }

        // Rust struct update syntax: Type { field: val, ..base }
        if let Some(base) = spread_base {
            Some(quote! { #type_ident { #(#fields,)* ..#base } })
        } else {
            Some(quote! { #type_ident { #(#fields),* } })
        }
    }
}
