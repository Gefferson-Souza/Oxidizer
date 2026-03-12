//! Function declaration transpilation.
//!
//! Converts TypeScript function declarations to Rust function items.
//! Handles: sync/async, return types, parameter mapping, Result wrapping.

use quote::{format_ident, quote};
use swc_ecma_ast::{FnDecl, Pat};

use super::helpers::to_snake_case;
use super::interface::RustGenerator;
use super::type_mapper::{map_ts_type, unwrap_promise_type};

impl RustGenerator {
    /// Convert a TypeScript function declaration to a Rust function.
    pub fn process_fn_decl(&mut self, n: &FnDecl) {
        let fn_name = to_snake_case(&n.ident.sym);
        let fn_ident = format_ident!("{}", fn_name);

        // Check if async
        let is_async = n.function.is_async;

        // Extract parameters
        let mut params = Vec::new();
        for param in &n.function.params {
            if let Pat::Ident(ident_pat) = &param.pat {
                let param_name = format_ident!("{}", ident_pat.sym.to_string());
                let param_type = map_ts_type(ident_pat.type_ann.as_ref());
                params.push(quote! { #param_name: #param_type });
            }
        }

        // Extract return type - unwrap Promise<T> for async functions
        let return_type = if is_async {
            if n.function.return_type.is_none() {
                quote! { Result<(), crate::AppError> }
            } else {
                let inner = unwrap_promise_type(n.function.return_type.as_ref());
                quote! { Result<#inner, crate::AppError> }
            }
        } else if n.function.return_type.is_none() {
            quote! { () }
        } else {
            map_ts_type(n.function.return_type.as_ref())
        };

        // Check if void
        let is_void = if n.function.return_type.is_none() {
            true
        } else {
            super::type_mapper::is_void_or_promise_void(n.function.return_type.as_deref())
        };

        // Convert body
        let mut body_stmts = Vec::new();
        if let Some(block_stmt) = &n.function.body {
            if is_async {
                for stmt in &block_stmt.stmts {
                    body_stmts.push(self.convert_stmt_recursive(stmt, &mut |ret_stmt| {
                        if let Some(arg) = &ret_stmt.arg {
                            let expr = self.convert_expr(arg);
                            if !is_void && matches!(arg.as_ref(), swc_ecma_ast::Expr::Object(_)) {
                                quote! {
                                    return Ok(serde_json::from_value(#expr).unwrap_or_default());
                                }
                            } else {
                                quote! { return Ok(#expr); }
                            }
                        } else {
                            quote! { return Ok(()); }
                        }
                    }));
                }
            } else {
                for stmt in &block_stmt.stmts {
                    body_stmts.push(self.convert_stmt_recursive(stmt, &mut |ret_stmt| {
                        if let Some(arg) = &ret_stmt.arg {
                            let expr = self.convert_expr(arg);
                            if !is_void && matches!(arg.as_ref(), swc_ecma_ast::Expr::Object(_)) {
                                quote! {
                                    return serde_json::from_value(#expr).unwrap_or_default();
                                }
                            } else {
                                quote! { return #expr; }
                            }
                        } else {
                            quote! { return; }
                        }
                    }));
                }
            }
        }

        let vis = if self.is_exporting {
            quote! { pub }
        } else {
            quote! {}
        };

        let generics = if let Some(type_params) = &n.function.type_params {
            let params: Vec<_> = type_params
                .params
                .iter()
                .map(|p| {
                    let name = p.name.sym.to_string();
                    let ident = format_ident!("{}", name);
                    quote! { #ident: serde::de::DeserializeOwned + serde::Serialize + Clone }
                })
                .collect();
            quote! { <#(#params),*> }
        } else {
            quote! {}
        };

        let fn_def = if is_async {
            let fallback = if is_void {
                quote! { Ok(()) }
            } else {
                quote! {}
            };

            quote! {
                #vis async fn #fn_ident #generics (#(#params),*) -> #return_type {
                    #(#body_stmts)*
                    #fallback
                }
            }
        } else {
            quote! {
                #vis fn #fn_ident #generics (#(#params),*) -> #return_type {
                    #(#body_stmts)*
                }
            }
        };

        self.code.push_str(&fn_def.to_string());
        self.code.push('\n');
    }
}
