use quote::{format_ident, quote};
use swc_ecma_ast::ClassDecl;
use tyrus_decorator_kinds::DecoratorKind;

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;
use crate::decorators;

/// Extracted controller decorator information.
pub(crate) struct ControllerInfo {
    pub is_controller: bool,
    pub controller_path: String,
    /// Guard class names from `@UseGuards(Guard1, Guard2)`
    pub guard_names: Vec<String>,
}

/// Extracts `@Controller("/path")` and `@UseGuards(...)` decorator info from
/// a class via the shared [`crate::decorators::DecoratorRegistry`]. The
/// per-decorator parsing logic lives in the handler files
/// (`decorators/controller.rs`, `decorators/use_guards.rs`); this function
/// is the bridge that exposes the registry's [`ClassDecoratorContext`] to
/// legacy callers.
pub(crate) fn extract_controller_info(n: &ClassDecl) -> ControllerInfo {
    let mut ctx = decorators::ClassDecoratorContext::default();
    decorators::shared_registry().apply_class_decorators(n, &mut ctx);
    ControllerInfo {
        is_controller: ctx.is_controller,
        controller_path: ctx.controller_path,
        guard_names: ctx.guard_names,
    }
}

/// Finds the `canActivate()` method in a class, if present.
fn find_can_activate_method(n: &ClassDecl) -> Option<&swc_ecma_ast::ClassMethod> {
    n.class.body.iter().find_map(|member| {
        if let swc_ecma_ast::ClassMember::Method(method) = member {
            if let Some(ident) = method.key.as_ident() {
                if ident.sym.as_ref() == "canActivate" {
                    return Some(method);
                }
            }
        }
        None
    })
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
            let fn_name = format_ident!("{}_middleware", to_snake_case(name));
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
///
/// The HTTP verb name is mapped to its `axum::routing::*` constructor via
/// [`DecoratorKind::axum_routing_fn_name`] — the same source of truth
/// `extract_method_decorators` uses, eliminating the previous duplicate
/// match on `"Get"` / `"Post"` / etc. that existed in both this module
/// and `class/method.rs`.
fn build_route_calls(
    routes: &[(String, String, String)],
    controller_path: &str,
) -> Vec<proc_macro2::TokenStream> {
    routes
        .iter()
        .map(|(method_name, http_method, path)| {
            build_single_route(method_name, http_method, path, controller_path)
        })
        .collect()
}

/// Emits a single `.route(path, axum::routing::verb(Self::handler))` call,
/// or a `compile_error!` token if the verb is unknown (which would indicate
/// an internal inconsistency between the registry and this dispatcher).
fn build_single_route(
    method_name: &str,
    http_method: &str,
    path: &str,
    controller_path: &str,
) -> proc_macro2::TokenStream {
    let Some(axum_fn_name) =
        DecoratorKind::from_name(http_method).and_then(DecoratorKind::axum_routing_fn_name)
    else {
        let msg = format!("Tyrus: unknown HTTP method decorator '{http_method}'");
        return quote! { compile_error!(#msg); };
    };
    let method_ident = format_ident!("{}", method_name);
    let axum_method = format_ident!("{}", axum_fn_name);
    let full_path = combine_paths(controller_path, path);
    quote! { .route(#full_path, axum::routing::#axum_method(Self::#method_ident)) }
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

/// Common HTTP status codes mapped to their `axum::http::StatusCode`
/// constants. Adding support for a new code is a one-line table edit;
/// uncovered codes generate a `compile_error!` rather than a runtime
/// `unwrap_or` fallback.
const STATUS_CODES: &[(u16, &str)] = &[
    (100, "CONTINUE"),
    (200, "OK"),
    (201, "CREATED"),
    (202, "ACCEPTED"),
    (204, "NO_CONTENT"),
    (301, "MOVED_PERMANENTLY"),
    (302, "FOUND"),
    (304, "NOT_MODIFIED"),
    (400, "BAD_REQUEST"),
    (401, "UNAUTHORIZED"),
    (403, "FORBIDDEN"),
    (404, "NOT_FOUND"),
    (405, "METHOD_NOT_ALLOWED"),
    (409, "CONFLICT"),
    (410, "GONE"),
    (422, "UNPROCESSABLE_ENTITY"),
    (429, "TOO_MANY_REQUESTS"),
    (500, "INTERNAL_SERVER_ERROR"),
    (502, "BAD_GATEWAY"),
    (503, "SERVICE_UNAVAILABLE"),
    (504, "GATEWAY_TIMEOUT"),
];

/// Maps an HTTP status code to its `axum::http::StatusCode` constant. Codes
/// outside [`STATUS_CODES`] expand to a `compile_error!` token — generated
/// Rust never carries an `unwrap_or` fallback for status codes.
pub(crate) fn map_status_code(code: u16) -> proc_macro2::TokenStream {
    use quote::quote;
    if let Some((_, name)) = STATUS_CODES.iter().find(|(c, _)| *c == code) {
        let ident = format_ident!("{}", name);
        return quote! { axum::http::StatusCode::#ident };
    }
    let msg = format!(
        "Tyrus: @HttpCode({code}) is not in the supported status code table; \
         add it to STATUS_CODES in convert/class/routing.rs or use a standard code"
    );
    quote! { compile_error!(#msg) }
}

/// Builds the doc comment for a handler method.
pub(crate) fn build_doc_comment(
    http_method: Option<&String>,
    route_path: &str,
) -> proc_macro2::TokenStream {
    use quote::quote;
    let Some(m) = http_method else {
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
        let can_activate = find_can_activate_method(n)?;
        let fn_name = format_ident!("{}_middleware", to_snake_case(class_name));
        let body_stmts = self.convert_guard_body(can_activate);

        Some(quote! {
            async fn #fn_name(
                headers: axum::http::HeaderMap,
                request: axum::http::Request<axum::body::Body>,
                next: axum::middleware::Next,
            ) -> Result<axum::response::Response, axum::http::StatusCode> {
                #(#body_stmts)*
                Ok(next.run(request).await)
            }
        })
    }

    /// Converts the `canActivate()` method body into statements for the middleware.
    fn convert_guard_body(
        &self,
        method: &swc_ecma_ast::ClassMethod,
    ) -> Vec<proc_macro2::TokenStream> {
        let Some(body) = &method.function.body else {
            return Vec::new();
        };
        body.stmts
            .iter()
            .map(|s| self.convert_guard_stmt(s))
            .collect()
    }

    /// Converts a guard statement, rewriting `return false` → 401 response.
    fn convert_guard_stmt(&self, stmt: &swc_ecma_ast::Stmt) -> proc_macro2::TokenStream {
        // Intercept `return false` → Err(UNAUTHORIZED)
        if let swc_ecma_ast::Stmt::Return(ret) = stmt {
            if let Some(arg) = &ret.arg {
                if let swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Bool(b)) = &**arg {
                    if !b.value {
                        return quote! {
                            return Err(axum::http::StatusCode::UNAUTHORIZED);
                        };
                    }
                    // `return true` → no-op (fall through to Ok(next.run(...)))
                    return quote! {};
                }
            }
        }
        self.convert_stmt(stmt)
    }

    /// Emits controller-specific code: `FromRequestParts` impl, router method,
    /// and records the controller metadata.
    // Why allowed: Axum router emission threads multiple unrelated context
    // inputs (struct ident, class metadata, controller info, route table,
    // impl-items accumulator). Rule 4 trade-off explicitly acknowledged;
    // refactor to a `RoutingCtx` struct is tracked in #153 (file split of
    // routing.rs). Until then, the override is the smaller compromise.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_status_code_emits_constant_for_known_code() {
        let token = map_status_code(200);
        assert!(token.to_string().contains("StatusCode :: OK"));
    }

    #[test]
    fn map_status_code_covers_full_table() {
        for (code, name) in STATUS_CODES {
            let token = map_status_code(*code);
            let s = token.to_string();
            assert!(
                s.contains(name),
                "code {code} expected to emit {name}, got: {s}"
            );
            assert!(
                !s.contains("compile_error"),
                "code {code} unexpectedly emitted compile_error"
            );
            assert!(
                !s.contains("unwrap_or"),
                "code {code} regressed: unwrap_or should not appear in generated code"
            );
        }
    }

    #[test]
    fn map_status_code_emits_compile_error_for_unknown_code() {
        for code in [0_u16, 999, 12345_u16.saturating_mul(1)] {
            let token = map_status_code(code);
            let s = token.to_string();
            assert!(
                s.contains("compile_error"),
                "code {code} should emit compile_error, got: {s}"
            );
            assert!(
                !s.contains("unwrap_or"),
                "code {code} regressed: generated code must not contain unwrap_or"
            );
        }
    }

    #[test]
    fn build_single_route_known_verb_emits_axum_routing() {
        let token = build_single_route("find_all", "Get", "", "/users");
        let s = token.to_string();
        assert!(s.contains("axum :: routing :: get"));
        assert!(s.contains("\"/users\""));
        assert!(s.contains("Self :: find_all"));
    }

    #[test]
    fn build_single_route_unknown_verb_emits_compile_error() {
        let token = build_single_route("foo", "BogusVerb", "", "/x");
        let s = token.to_string();
        assert!(
            s.contains("compile_error"),
            "unknown verb should emit compile_error, got: {s}"
        );
        assert!(
            s.contains("BogusVerb"),
            "compile_error should name the verb"
        );
    }
}
