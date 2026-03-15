use quote::{format_ident, quote};
use swc_ecma_ast::{ClassDecl, Expr, Lit};

use crate::convert::interface::RustGenerator;

/// Extracted controller decorator information.
pub(crate) struct ControllerInfo {
    pub is_controller: bool,
    pub controller_path: String,
    /// Guard class names from `@UseGuards(Guard1, Guard2)`
    pub guard_names: Vec<String>,
}

/// Extracts `@Controller("/path")` and `@UseGuards(...)` decorator info from a class.
pub(crate) fn extract_controller_info(n: &ClassDecl) -> ControllerInfo {
    let mut is_controller = false;
    let mut controller_path = String::new();
    let mut guard_names = Vec::new();

    for decorator in &n.class.decorators {
        if let Expr::Call(call) = &*decorator.expr {
            if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                if let Expr::Ident(ident) = &**expr {
                    match ident.sym.as_ref() {
                        "Controller" => {
                            is_controller = true;
                            if let Some(arg) = call.args.first() {
                                if let Expr::Lit(Lit::Str(s)) = &*arg.expr {
                                    controller_path =
                                        s.value.as_str().unwrap_or_default().to_string();
                                }
                            }
                        }
                        "UseGuards" => {
                            extract_guard_args(call, &mut guard_names);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    ControllerInfo {
        is_controller,
        controller_path,
        guard_names,
    }
}

/// Extracts guard class names from `@UseGuards(Guard1, Guard2)` arguments.
fn extract_guard_args(call: &swc_ecma_ast::CallExpr, guard_names: &mut Vec<String>) {
    for arg in &call.args {
        if let Expr::Ident(ident) = &*arg.expr {
            guard_names.push(ident.sym.to_string());
        }
    }
}

/// Generates the `FromRequestParts` implementation for a controller struct.
pub(crate) fn generate_from_request_parts_impl(
    struct_name: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    quote! {
        #[axum::async_trait]
        impl<S> axum::extract::FromRequestParts<S> for #struct_name
        where S: Send + Sync
        {
            type Rejection = axum::http::StatusCode;
            async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
                parts.extensions
                    .get::<std::sync::Arc<Self>>()
                    .cloned()
                    .map(|arc| arc.as_ref().clone())
                    .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Generates the `router()` method with route calls and optional guard layers.
pub(crate) fn generate_router_method(
    routes: &[(String, String, String)],
    controller_path: &str,
    guard_names: &[String],
) -> proc_macro2::TokenStream {
    let route_calls = build_route_calls(routes, controller_path);

    let layer_calls: Vec<_> = guard_names
        .iter()
        .map(|name| {
            let fn_name =
                format_ident!("{}_middleware", super::super::helpers::to_snake_case(name));
            quote! { .layer(axum::middleware::from_fn(#fn_name)) }
        })
        .collect();

    quote! {
        pub fn router(state: std::sync::Arc<Self>) -> axum::Router {
            axum::Router::new()
                #(#route_calls)*
                #(#layer_calls)*
                .with_state(state)
        }
    }
}

/// Builds individual `.route()` calls from route metadata.
fn build_route_calls(
    routes: &[(String, String, String)],
    controller_path: &str,
) -> Vec<proc_macro2::TokenStream> {
    routes
        .iter()
        .map(|(method_name, http_method, path)| {
            let method_ident = format_ident!("{}", method_name);
            let axum_method = match http_method.as_str() {
                "Get" => quote! { get },
                "Post" => quote! { post },
                "Put" => quote! { put },
                "Delete" => quote! { delete },
                "Patch" => quote! { patch },
                _ => quote! { get },
            };
            let full_path = combine_paths(controller_path, path);
            quote! { .route(#full_path, axum::routing::#axum_method(Self::#method_ident)) }
        })
        .collect()
}

/// Combines controller path and method path into a full route path.
fn combine_paths(controller_path: &str, method_path: &str) -> String {
    let full = if controller_path.is_empty() {
        method_path.to_string()
    } else {
        let c = controller_path.trim_matches('/');
        let m = method_path.trim_matches('/');
        if m.is_empty() {
            format!("/{c}")
        } else {
            format!("/{c}/{m}")
        }
    };
    if full.starts_with('/') {
        full
    } else {
        format!("/{full}")
    }
}

/// Maps an HTTP status code to its axum StatusCode constant.
pub(crate) fn map_status_code(code: u16) -> proc_macro2::TokenStream {
    use quote::quote;
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

/// Builds the doc comment for a handler method.
pub(crate) fn build_doc_comment(
    http_method: &Option<String>,
    route_path: &str,
) -> proc_macro2::TokenStream {
    use quote::quote;
    let Some(m) = http_method.as_ref() else {
        return quote! {};
    };
    let method_str = m.to_uppercase();
    let route = if route_path.is_empty() {
        "/".to_string()
    } else {
        route_path.to_string()
    };
    quote! { #[doc = concat!("Route: ", #method_str, " ", #route)] }
}

impl RustGenerator {
    /// Checks if a class is a guard (has `canActivate()`) and emits middleware.
    pub(crate) fn try_emit_guard_middleware(
        &self,
        n: &ClassDecl,
        class_name: &str,
    ) -> Option<proc_macro2::TokenStream> {
        let can_activate = n.class.body.iter().find_map(|member| {
            if let swc_ecma_ast::ClassMember::Method(method) = member {
                if let Some(ident) = method.key.as_ident() {
                    if ident.sym.as_ref() == "canActivate" {
                        return Some(method);
                    }
                }
            }
            None
        })?;

        let fn_name = format_ident!(
            "{}_middleware",
            super::super::helpers::to_snake_case(class_name)
        );

        let body_stmts = self.convert_guard_body(can_activate);

        Some(quote! {
            async fn #fn_name(
                headers: axum::http::HeaderMap,
                request: axum::http::Request<axum::body::Body>,
                next: axum::middleware::Next,
            ) -> Result<axum::response::Response, axum::http::StatusCode> {
                let can_activate = (|| -> bool {
                    #(#body_stmts)*
                })();
                if can_activate {
                    Ok(next.run(request).await)
                } else {
                    Err(axum::http::StatusCode::UNAUTHORIZED)
                }
            }
        })
    }

    /// Converts the `canActivate()` method body into statements for the middleware.
    fn convert_guard_body(
        &self,
        method: &swc_ecma_ast::ClassMethod,
    ) -> Vec<proc_macro2::TokenStream> {
        let Some(body) = &method.function.body else {
            return vec![quote! { true }];
        };
        body.stmts.iter().map(|s| self.convert_stmt(s)).collect()
    }

    /// Emits controller-specific code: `FromRequestParts` impl, router method,
    /// and records the controller metadata.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_controller_routing(
        &mut self,
        struct_name: &proc_macro2::Ident,
        class_name: &str,
        controller_info: &ControllerInfo,
        routes: &[(String, String, String)],
        impl_items: &mut Vec<proc_macro2::TokenStream>,
    ) {
        let from_request_impl = generate_from_request_parts_impl(struct_name);
        self.code.push_str(&from_request_impl.to_string());
        self.code.push('\n');

        let router_method = generate_router_method(
            routes,
            &controller_info.controller_path,
            &controller_info.guard_names,
        );
        impl_items.push(router_method);

        // Add to metadata
        self.controllers.push(crate::ControllerMetadata {
            struct_name: class_name.to_string(),
            route_path: controller_info.controller_path.clone(),
        });
    }
}
