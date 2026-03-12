use quote::{format_ident, quote};
use swc_ecma_ast::{ClassDecl, Expr, Lit};

use crate::convert::interface::RustGenerator;

/// Extracted controller decorator information.
pub(crate) struct ControllerInfo {
    pub is_controller: bool,
    pub controller_path: String,
}

/// Extracts `@Controller("/path")` decorator information from a class declaration.
pub(crate) fn extract_controller_info(n: &ClassDecl) -> ControllerInfo {
    let mut is_controller = false;
    let mut controller_path = String::new();

    for decorator in &n.class.decorators {
        if let Expr::Call(call) = &*decorator.expr {
            if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                if let Expr::Ident(ident) = &**expr {
                    if ident.sym == "Controller" {
                        is_controller = true;
                        if let Some(arg) = call.args.first() {
                            if let Expr::Lit(Lit::Str(s)) = &*arg.expr {
                                controller_path = s.value.as_str().unwrap_or_default().to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    ControllerInfo {
        is_controller,
        controller_path,
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

/// Generates the `router()` method and route calls for a controller.
pub(crate) fn generate_router_method(
    routes: &[(String, String, String)],
    controller_path: &str,
) -> proc_macro2::TokenStream {
    let mut route_calls = Vec::new();
    for (method_name, http_method, path) in routes {
        let method_ident = format_ident!("{}", method_name);
        let axum_method = match http_method.as_str() {
            "Get" => quote! { get },
            "Post" => quote! { post },
            "Put" => quote! { put },
            "Delete" => quote! { delete },
            "Patch" => quote! { patch },
            _ => quote! { get },
        };

        // Combine controller path and method path
        // Controller: "cats", Method: "/" -> "/cats"
        // Controller: "cats", Method: "/:id" -> "/cats/:id"

        let full_path = if controller_path.is_empty() {
            path.clone()
        } else {
            let c_path = controller_path.trim_matches('/');
            let m_path = path.trim_matches('/');
            if m_path.is_empty() {
                format!("/{}", c_path)
            } else {
                format!("/{}/{}", c_path, m_path)
            }
        };

        // Ensure starts with /
        let full_path = if full_path.starts_with('/') {
            full_path
        } else {
            format!("/{}", full_path)
        };

        route_calls.push(quote! {
            .route(#full_path, axum::routing::#axum_method(Self::#method_ident))
        });
    }

    quote! {
        pub fn router() -> axum::Router {
            axum::Router::new()
                #(#route_calls)*
        }
    }
}

impl RustGenerator {
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

        let router_method = generate_router_method(routes, &controller_info.controller_path);
        impl_items.push(router_method);

        // Add to metadata
        self.controllers.push(crate::ControllerMetadata {
            struct_name: class_name.to_string(),
            route_path: controller_info.controller_path.clone(),
        });
    }
}
