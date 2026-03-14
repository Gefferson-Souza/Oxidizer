use quote::{format_ident, quote};
use swc_ecma_ast::{Expr, Lit, Pat};

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;
use crate::convert::type_mapper::{is_optional_type, map_ts_type};

/// Extracted decorator metadata from a class method.
struct MethodDecorators {
    http_method: Option<String>,
    route_path: String,
    http_code: Option<u16>,
}

/// Context flags computed from decorators and method properties.
struct MethodContext {
    is_handler: bool,
    is_static: bool,
    is_async: bool,
    http_code: Option<u16>,
    returns_option: bool,
}

/// Scans method decorators for @Get/@Post/@Put/@Delete/@Patch and @HttpCode.
fn extract_method_decorators(method: &swc_ecma_ast::ClassMethod) -> MethodDecorators {
    let mut http_method = None;
    let mut route_path = String::new();
    let mut http_code: Option<u16> = None;

    for decorator in &method.function.decorators {
        if let Expr::Call(call) = &*decorator.expr {
            if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                if let Expr::Ident(ident) = &**expr {
                    extract_single_decorator(
                        ident.sym.as_str(),
                        call,
                        &mut http_method,
                        &mut route_path,
                        &mut http_code,
                    );
                }
            }
        }
    }

    MethodDecorators {
        http_method,
        route_path,
        http_code,
    }
}

/// Processes a single decorator call to extract HTTP method or status code.
fn extract_single_decorator(
    name: &str,
    call: &swc_ecma_ast::CallExpr,
    http_method: &mut Option<String>,
    route_path: &mut String,
    http_code: &mut Option<u16>,
) {
    if matches!(name, "Get" | "Post" | "Put" | "Delete" | "Patch") {
        *http_method = Some(name.to_string());
        if let Some(arg) = call.args.first() {
            if let Expr::Lit(Lit::Str(s)) = &*arg.expr {
                *route_path = s.value.as_str().unwrap_or_default().to_string();
            }
        }
    } else if name == "HttpCode" {
        if let Some(arg) = call.args.first() {
            if let Expr::Lit(Lit::Num(num)) = &*arg.expr {
                *http_code = Some(num.value as u16);
            }
        }
    }
}

/// Builds the self parameter token based on method kind and mutation analysis.
fn build_self_param(
    method: &swc_ecma_ast::ClassMethod,
    ctx: &MethodContext,
    is_service_or_controller: bool,
) -> Option<proc_macro2::TokenStream> {
    if ctx.is_static {
        return None;
    }

    if ctx.is_handler {
        return Some(quote! { self });
    }

    if is_service_or_controller {
        return Some(quote! { &self });
    }

    let mutates = method
        .function
        .body
        .as_ref()
        .is_some_and(|body| RustGenerator::body_mutates_self(&body.stmts));

    if mutates {
        Some(quote! { &mut self })
    } else {
        Some(quote! { &self })
    }
}

/// Converts a single method parameter, handling @Body/@Param/@Query decorators.
fn convert_single_param(param: &swc_ecma_ast::Param) -> Option<proc_macro2::TokenStream> {
    let Pat::Ident(ident) = &param.pat else {
        return None;
    };

    let param_name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
    let param_type = map_ts_type(ident.type_ann.as_ref());
    let decorator = find_param_decorator(param);

    let tokens = match decorator.as_deref() {
        Some("Body") => {
            quote! { axum::Json(#param_name): axum::Json<#param_type> }
        }
        Some("Param") => {
            quote! { axum::extract::Path(#param_name): axum::extract::Path<#param_type> }
        }
        Some("Query") => {
            quote! { axum::extract::Query(#param_name): axum::extract::Query<#param_type> }
        }
        _ => {
            quote! { #param_name: #param_type }
        }
    };
    Some(tokens)
}

/// Finds the first NestJS parameter decorator (@Body, @Param, @Query) on a param.
fn find_param_decorator(param: &swc_ecma_ast::Param) -> Option<String> {
    for decorator in &param.decorators {
        if let Expr::Call(call) = &*decorator.expr {
            if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                if let Expr::Ident(dec_ident) = &**expr {
                    let dec_name = dec_ident.sym.as_ref();
                    if matches!(dec_name, "Body" | "Param" | "Query") {
                        return Some(dec_name.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Builds the full parameter list including self and method params.
fn build_method_params(
    method: &swc_ecma_ast::ClassMethod,
    ctx: &MethodContext,
    is_service_or_controller: bool,
) -> Vec<proc_macro2::TokenStream> {
    let mut params = Vec::new();

    if let Some(self_param) = build_self_param(method, ctx, is_service_or_controller) {
        params.push(self_param);
    }

    for param in &method.function.params {
        if let Some(p) = convert_single_param(param) {
            params.push(p);
        }
    }

    params
}

/// Computes the return type, wrapping in Json/StatusCode for handlers.
fn compute_return_type(
    method: &swc_ecma_ast::ClassMethod,
    ctx: &MethodContext,
) -> proc_macro2::TokenStream {
    let base = compute_base_return_type(method, ctx.is_handler);

    if !ctx.is_handler {
        return base;
    }

    wrap_handler_return_type(&base, ctx.http_code)
}

/// Computes the raw return type before handler wrapping.
fn compute_base_return_type(
    method: &swc_ecma_ast::ClassMethod,
    is_handler: bool,
) -> proc_macro2::TokenStream {
    if method.function.is_async {
        let inner = if method.function.return_type.is_none() {
            quote! { () }
        } else {
            super::super::type_mapper::unwrap_promise_type(method.function.return_type.as_ref())
        };

        if is_handler {
            inner
        } else {
            quote! { Result<#inner, crate::AppError> }
        }
    } else if method.function.return_type.is_none() {
        quote! { () }
    } else {
        map_ts_type(method.function.return_type.as_ref())
    }
}

/// Wraps a handler return type in Json<T> and optionally (StatusCode, T).
fn wrap_handler_return_type(
    base: &proc_macro2::TokenStream,
    http_code: Option<u16>,
) -> proc_macro2::TokenStream {
    let return_type_str = base.to_string();
    let inner_type = if return_type_str != "String" {
        quote! { axum::Json<#base> }
    } else {
        quote! { String }
    };

    if http_code.is_some() {
        quote! { Result<(axum::http::StatusCode, #inner_type), crate::AppError> }
    } else {
        quote! { Result<#inner_type, crate::AppError> }
    }
}

/// Maps an HTTP status code to its axum StatusCode constant.
fn map_status_code(code: u16) -> proc_macro2::TokenStream {
    match code {
        200 => quote! { axum::http::StatusCode::OK },
        201 => quote! { axum::http::StatusCode::CREATED },
        204 => quote! { axum::http::StatusCode::NO_CONTENT },
        301 => quote! { axum::http::StatusCode::MOVED_PERMANENTLY },
        400 => quote! { axum::http::StatusCode::BAD_REQUEST },
        401 => quote! { axum::http::StatusCode::UNAUTHORIZED },
        403 => quote! { axum::http::StatusCode::FORBIDDEN },
        404 => quote! { axum::http::StatusCode::NOT_FOUND },
        409 => quote! { axum::http::StatusCode::CONFLICT },
        500 => quote! { axum::http::StatusCode::INTERNAL_SERVER_ERROR },
        100..=999 => {
            quote! { axum::http::StatusCode::from_u16(#code).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR) }
        }
        _ => {
            quote! { compile_error!("Tyrus: @HttpCode value is not a valid HTTP status code (100-999)") }
        }
    }
}

/// Builds a handler return expression with Json wrapping and optional StatusCode.
fn build_handler_return(
    expr: &proc_macro2::TokenStream,
    return_type: &proc_macro2::TokenStream,
    http_code: Option<u16>,
) -> proc_macro2::TokenStream {
    let ret_str = return_type.to_string();
    let uses_json = ret_str.contains("axum :: Json");

    if let Some(code) = http_code {
        let status = map_status_code(code);
        if uses_json {
            quote! { return Ok((#status, axum::Json(#expr.into()))); }
        } else {
            quote! { return Ok((#status, #expr.into())); }
        }
    } else if uses_json {
        quote! { return Ok(axum::Json(#expr.into())); }
    } else {
        quote! { return Ok(#expr.into()); }
    }
}

/// Builds a return statement for an Option-returning method.
fn build_option_return(arg: &Expr, expr: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let is_null = matches!(arg, Expr::Lit(Lit::Null(_)));
    let is_undefined = matches!(arg, Expr::Ident(id) if id.sym.as_ref() == "undefined");

    if is_null || is_undefined {
        quote! { return None; }
    } else {
        quote! { return Some(#expr); }
    }
}

/// Builds body statements using the appropriate return handler for the method kind.
fn build_method_body(
    generator: &RustGenerator,
    method: &swc_ecma_ast::ClassMethod,
    return_type: &proc_macro2::TokenStream,
    ctx: &MethodContext,
) -> Vec<proc_macro2::TokenStream> {
    let mut body_stmts = Vec::new();
    let Some(body) = &method.function.body else {
        return body_stmts;
    };

    let needs_custom_return = ctx.is_handler || ctx.is_async || ctx.returns_option;

    if needs_custom_return {
        let mut return_handler = |ret: &swc_ecma_ast::ReturnStmt| -> proc_macro2::TokenStream {
            build_return_stmt(generator, ret, return_type, ctx)
        };
        for stmt in &body.stmts {
            body_stmts.push(generator.convert_stmt_recursive(stmt, &mut return_handler));
        }
    } else {
        for stmt in &body.stmts {
            body_stmts.push(generator.convert_stmt(stmt));
        }
    }

    body_stmts
}

/// Builds a single return statement based on method context.
fn build_return_stmt(
    generator: &RustGenerator,
    ret: &swc_ecma_ast::ReturnStmt,
    return_type: &proc_macro2::TokenStream,
    ctx: &MethodContext,
) -> proc_macro2::TokenStream {
    let Some(arg) = &ret.arg else {
        if ctx.returns_option {
            return quote! { return None; };
        }
        return quote! { return Ok(().into()); };
    };

    let expr = generator.convert_expr(arg);

    if ctx.is_handler {
        return build_handler_return(&expr, return_type, ctx.http_code);
    }

    if ctx.is_async {
        return quote! { return Ok(#expr); };
    }

    if ctx.returns_option {
        return build_option_return(arg, &expr);
    }

    quote! { return #expr; }
}

/// Builds the doc comment for a handler method.
fn build_doc_comment(http_method: &Option<String>, route_path: &str) -> proc_macro2::TokenStream {
    let Some(m) = http_method.as_ref() else {
        return quote! {};
    };

    let method_str = m.to_uppercase();
    let route = if route_path.is_empty() {
        "/".to_string()
    } else {
        route_path.to_string()
    };

    quote! {
        #[doc = concat!("Route: ", #method_str, " ", #route)]
    }
}

impl RustGenerator {
    /// Converts a class method into a Rust fn definition and optional route info.
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

        let decorators = extract_method_decorators(method);

        let ctx = MethodContext {
            is_handler: decorators.http_method.is_some(),
            is_static: method.is_static,
            is_async: method.function.is_async,
            http_code: decorators.http_code,
            returns_option: is_optional_type(method.function.return_type.as_deref()),
        };

        let params = build_method_params(method, &ctx, is_service_or_controller);
        let return_type = compute_return_type(method, &ctx);
        let body_stmts = build_method_body(self, method, &return_type, &ctx);

        let fn_keyword = if ctx.is_handler || ctx.is_async {
            quote! { async fn }
        } else {
            quote! { fn }
        };

        let doc_comment = build_doc_comment(&decorators.http_method, &decorators.route_path);

        let tokens = quote! {
            #doc_comment
            pub #fn_keyword #method_name(#(#params),*) -> #return_type {
                #(#body_stmts)*
            }
        };

        let route_info = decorators
            .http_method
            .map(|m| (method_name.to_string(), m, decorators.route_path));

        (tokens, route_info)
    }
}
