//! Function declaration transpilation.
//!
//! Converts TypeScript function declarations to Rust function items.
//! Handles: sync/async, return types, parameter mapping, Result wrapping.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{FnDecl, Function, Pat};

use super::helpers::to_snake_case;
use super::interface::RustGenerator;
use super::type_mapper::{map_ts_type, unwrap_promise_type};

impl RustGenerator {
    /// Convert a TypeScript function declaration to a Rust function.
    pub(crate) fn process_fn_decl(&mut self, n: &FnDecl) {
        let fn_name = to_snake_case(&n.ident.sym);
        let fn_ident = format_ident!("{}", fn_name);

        if fn_name == "main" {
            self.has_declared_main = true;
        }

        let is_async = n.function.is_async;
        let params = Self::extract_fn_params(&n.function);
        let return_type = Self::resolve_return_type(&n.function, is_async);
        let is_void = Self::is_void_return(&n.function);
        let body_stmts = self.build_fn_body(&n.function, is_async, is_void);
        let generics = Self::build_generics(&n.function);

        let vis = if self.is_exporting {
            quote! { pub }
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

    /// Extract parameters from a TypeScript function, mapping each to Rust syntax.
    fn extract_fn_params(function: &Function) -> Vec<TokenStream> {
        let mut params = Vec::new();
        for param in &function.params {
            match &param.pat {
                Pat::Ident(ident_pat) => {
                    let param_name = format_ident!("{}", ident_pat.sym.to_string());
                    let param_type = map_ts_type(ident_pat.type_ann.as_ref());
                    params.push(quote! { mut #param_name: #param_type });
                }
                Pat::Rest(rest_pat) => {
                    // ...args: T[] -> args: Vec<T>
                    if let Pat::Ident(ident) = &*rest_pat.arg {
                        let param_name = format_ident!("{}", ident.sym.to_string());
                        let param_type =
                            map_ts_type(rest_pat.type_ann.as_ref().or(ident.type_ann.as_ref()));
                        params.push(quote! { mut #param_name: #param_type });
                    }
                }
                _ => {}
            }
        }
        params
    }

    /// Resolve the Rust return type from a TS function, wrapping async in Result.
    fn resolve_return_type(function: &Function, is_async: bool) -> TokenStream {
        if is_async {
            if function.return_type.is_none() {
                quote! { Result<(), crate::AppError> }
            } else {
                let inner = unwrap_promise_type(function.return_type.as_ref());
                quote! { Result<#inner, crate::AppError> }
            }
        } else if function.return_type.is_none() {
            quote! { () }
        } else {
            map_ts_type(function.return_type.as_ref())
        }
    }

    /// Check whether the function has a void (or Promise<void>) return type.
    fn is_void_return(function: &Function) -> bool {
        if function.return_type.is_none() {
            return true;
        }
        super::type_mapper::is_void_or_promise_void(function.return_type.as_deref())
    }

    /// Convert the function body statements, wrapping returns in Ok() for async.
    fn build_fn_body(
        &self,
        function: &Function,
        is_async: bool,
        is_void: bool,
    ) -> Vec<TokenStream> {
        let Some(block_stmt) = &function.body else {
            return Vec::new();
        };

        block_stmt
            .stmts
            .iter()
            .map(|stmt| {
                self.convert_stmt_recursive(stmt, &mut |ret_stmt| {
                    Self::convert_return(self, ret_stmt, is_async, is_void)
                })
            })
            .collect()
    }

    /// Produce the return expression for a single ReturnStmt.
    fn convert_return(
        &self,
        ret_stmt: &swc_ecma_ast::ReturnStmt,
        is_async: bool,
        is_void: bool,
    ) -> TokenStream {
        let Some(arg) = &ret_stmt.arg else {
            return if is_async {
                quote! { return Ok(()); }
            } else {
                quote! { return; }
            };
        };

        let expr = self.convert_expr(arg);
        let is_object = !is_void && matches!(arg.as_ref(), swc_ecma_ast::Expr::Object(_));

        match (is_async, is_object) {
            (_, true) if is_async => quote! {
                return Ok(serde_json::from_value(#expr).unwrap_or_default());
            },
            (_, true) => quote! {
                return serde_json::from_value(#expr).unwrap_or_default();
            },
            (true, false) => quote! { return Ok(#expr); },
            (false, false) => quote! { return #expr; },
        }
    }

    /// Build generic type parameters with Serde + Clone bounds.
    fn build_generics(function: &Function) -> TokenStream {
        let Some(type_params) = &function.type_params else {
            return quote! {};
        };

        let params: Vec<_> = type_params
            .params
            .iter()
            .map(|p| {
                let ident = format_ident!("{}", p.name.sym.to_string());
                quote! { #ident: serde::de::DeserializeOwned + serde::Serialize + Clone }
            })
            .collect();

        quote! { <#(#params),*> }
    }
}
