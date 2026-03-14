use quote::{format_ident, quote};
use swc_ecma_ast::{Expr, Lit, Pat};

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;
use crate::convert::type_mapper::{is_optional_type, map_ts_type};

impl RustGenerator {
    pub(crate) fn convert_method(
        &self,
        method: &swc_ecma_ast::ClassMethod,
        is_service_or_controller: bool,
    ) -> (proc_macro2::TokenStream, Option<(String, String, String)>) {
        let method_name_str = if let Some(ident) = method.key.as_ident() {
            ident.sym.to_string()
        } else {
            return (quote! { /* unsupported method key */ }, None);
        };
        let method_name = format_ident!("{}", to_snake_case(&method_name_str));

        // Check for NestJS decorators (@Get, @Post, etc.)
        let mut http_method = None;
        let mut route_path = String::new();

        for decorator in &method.function.decorators {
            if let Expr::Call(call) = &*decorator.expr {
                if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                    if let Expr::Ident(ident) = &**expr {
                        let name = ident.sym.as_str();
                        if matches!(name, "Get" | "Post" | "Put" | "Delete" | "Patch") {
                            http_method = Some(name.to_string());
                            // Extract route path if present
                            if let Some(arg) = call.args.first() {
                                if let Expr::Lit(Lit::Str(s)) = &*arg.expr {
                                    route_path = s.value.as_str().unwrap_or_default().to_string();
                                }
                            }
                        }
                    }
                }
            }
        }

        let is_handler = http_method.is_some();

        // Build parameters
        let mut params = Vec::new();

        // Handle self parameter — static methods have no self
        let is_static = method.is_static;
        if !is_static {
            if is_handler {
                params.push(quote! { self });
            } else if is_service_or_controller {
                params.push(quote! { &self });
            } else {
                let mutates = method
                    .function
                    .body
                    .as_ref()
                    .is_some_and(|body| Self::body_mutates_self(&body.stmts));
                if mutates {
                    params.push(quote! { &mut self });
                } else {
                    params.push(quote! { &self });
                }
            }
        }

        for param in &method.function.params {
            if let Pat::Ident(ident) = &param.pat {
                let param_name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                let param_type = map_ts_type(ident.type_ann.as_ref());

                // Check for @Body decorator on parameters
                let mut is_body = false;

                // Correctly check decorators on the Param node
                for decorator in &param.decorators {
                    if let Expr::Call(call) = &*decorator.expr {
                        if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                            if let Expr::Ident(ident) = &**expr {
                                if ident.sym == "Body" {
                                    is_body = true;
                                }
                            }
                        }
                    }
                }

                if is_body {
                    params.push(quote! { axum::Json(#param_name): axum::Json<#param_type> });
                } else {
                    params.push(quote! { #param_name: #param_type });
                }
            }
        }

        let mut return_type = if method.function.is_async {
            let inner = if method.function.return_type.is_none() {
                quote! { () }
            } else {
                super::super::type_mapper::unwrap_promise_type(method.function.return_type.as_ref())
            };

            if !is_handler {
                quote! { Result<#inner, crate::AppError> }
            } else {
                inner
            }
        } else if method.function.return_type.is_none() {
            quote! { () }
        } else {
            map_ts_type(method.function.return_type.as_ref())
        };

        // If it's a handler, wrap return type in Json unless it's String
        // Wrap handler return type in Result
        if is_handler {
            let return_type_str = return_type.to_string();
            let inner_type = if return_type_str != "String" {
                quote! { axum::Json<#return_type> }
            } else {
                quote! { String }
            };
            return_type = quote! { Result<#inner_type, crate::AppError> };
        }

        // Check if this method returns Option<T> (from T | null union)
        let returns_option = is_optional_type(method.function.return_type.as_deref());

        // Convert body
        let mut body_stmts = Vec::new();
        if let Some(body) = &method.function.body {
            // Define return handler
            let mut return_handler = |ret: &swc_ecma_ast::ReturnStmt| -> proc_macro2::TokenStream {
                if let Some(arg) = &ret.arg {
                    let expr = self.convert_expr(arg);

                    if is_handler {
                        // Check if we wrapped the return type in Json (inside Result)
                        let ret_str = return_type.to_string();
                        let uses_json = ret_str.contains("axum :: Json");

                        if uses_json {
                            quote! { return Ok(axum::Json(#expr.into())); }
                        } else {
                            quote! { return Ok(#expr.into()); }
                        }
                    } else if method.function.is_async {
                        // For async methods, wrap in Ok
                        quote! { return Ok(#expr); }
                    } else if returns_option {
                        // For methods returning Option<T>, wrap non-null returns in Some()
                        let is_null = matches!(&**arg, Expr::Lit(Lit::Null(_)));
                        let is_undefined =
                            matches!(&**arg, Expr::Ident(id) if id.sym.as_ref() == "undefined");
                        if is_null || is_undefined {
                            quote! { return None; }
                        } else {
                            quote! { return Some(#expr); }
                        }
                    } else {
                        quote! { return #expr; }
                    }
                } else if returns_option {
                    quote! { return None; }
                } else {
                    quote! { return Ok(().into()); } // For handlers returning void?
                }
            };

            for stmt in &body.stmts {
                if is_handler || method.function.is_async || returns_option {
                    body_stmts.push(self.convert_stmt_recursive(stmt, &mut return_handler));
                } else {
                    body_stmts.push(self.convert_stmt(stmt));
                }
            }
        }

        let fn_keyword = if is_handler || method.function.is_async {
            quote! { async fn }
        } else {
            quote! { fn }
        };

        let doc_comment = if is_handler {
            let method_str = if let Some(m) = http_method.as_ref() {
                m.to_uppercase()
            } else {
                "GET".to_string() // Default or unreachable if guarded
            };
            let route = if route_path.is_empty() {
                "/".to_string()
            } else {
                route_path.clone()
            };
            quote! {
                #[doc = concat!("Route: ", #method_str, " ", #route)]
            }
        } else {
            quote! {}
        };

        let tokens = quote! {
            #doc_comment
            pub #fn_keyword #method_name(#(#params),*) -> #return_type {
                #(#body_stmts)*
            }
        };

        let route_info = http_method.map(|method| (method_name.to_string(), method, route_path));

        (tokens, route_info)
    }
}
