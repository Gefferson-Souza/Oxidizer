//! Decorator handler registry.
//!
//! This module is the architectural answer to the prior approach where every
//! NestJS decorator required hardcoded match arms scattered across
//! `class/mod.rs`, `class/method.rs`, `class/routing.rs`, and the analyzer.
//!
//! ## Design
//!
//! Three traits, one per scope:
//!
//! - [`ClassDecoratorHandler`] — runs over a `ClassDecl`, mutates a [`ClassDecoratorContext`]
//! - [`MethodDecoratorHandler`] — runs over a `ClassMethod`, mutates a [`MethodDecoratorContext`]
//! - [`ParamDecoratorHandler`] — runs over a `Param`, emits an Axum extractor token
//!
//! A [`DecoratorRegistry`] dispatches a decorator's identifier to the right
//! handler via [`tyrus_decorator_kinds::DecoratorKind`]. Adding a new decorator
//! means: add one variant to `DecoratorKind`, write one handler file, register
//! it in [`default_registry`]. No hot-path file is touched.
//!
//! ## What's here today (PR #2)
//!
//! Class-level (`@Controller`, `@UseGuards`) and method-level (`@Get`/`@Post`/
//! `@Put`/`@Delete`/`@Patch` + `@HttpCode`) handlers are wired. Param-level
//! handlers (`@Body`/`@Param`/`@Query`) land in PR #3.

pub(crate) mod controller;
pub(crate) mod http_code;
pub(crate) mod http_method;
pub(crate) mod use_guards;

use std::collections::HashMap;
use std::sync::OnceLock;
use swc_ecma_ast::{CallExpr, ClassDecl, ClassMethod, Param};
use tyrus_decorator_kinds::DecoratorKind;

/// Mutable bag of class-level metadata that handlers populate.
///
/// `ControllerHandler` flips `is_controller` and writes `controller_path`;
/// `UseGuardsHandler` appends to `guard_names`. Code outside this module
/// reads the resulting struct — it never observes which handler wrote what.
#[derive(Debug, Default, Clone)]
pub(crate) struct ClassDecoratorContext {
    pub(crate) is_controller: bool,
    pub(crate) controller_path: String,
    pub(crate) guard_names: Vec<String>,
}

/// Mutable bag of method-level metadata. `HttpMethodHandler` writes
/// `http_method` + `route_path`; `HttpCodeHandler` writes `http_code`.
#[derive(Debug, Default, Clone)]
pub(crate) struct MethodDecoratorContext {
    /// The [`DecoratorKind`] of the HTTP verb decorator, if any.
    /// `None` ⇒ method is not an HTTP handler.
    pub(crate) http_method: Option<DecoratorKind>,
    pub(crate) route_path: String,
    pub(crate) http_code: Option<u16>,
}

/// Handler for class-level decorators (`@Controller`, `@UseGuards`, ...).
pub(crate) trait ClassDecoratorHandler: Send + Sync {
    fn kind(&self) -> DecoratorKind;
    fn apply(&self, class: &ClassDecl, call: &CallExpr, ctx: &mut ClassDecoratorContext);
}

/// Handler for method-level decorators (`@Get`, `@Post`, `@HttpCode`, ...).
pub(crate) trait MethodDecoratorHandler: Send + Sync {
    fn kind(&self) -> DecoratorKind;
    fn apply(&self, method: &ClassMethod, call: &CallExpr, ctx: &mut MethodDecoratorContext);
}

/// Handler for param-level decorators (`@Body`, `@Param`, `@Query`, ...).
/// Wired in PR #3. `emit_extractor` returns the Axum extractor token to
/// substitute for the parameter binding.
#[allow(dead_code)]
pub(crate) trait ParamDecoratorHandler: Send + Sync {
    fn kind(&self) -> DecoratorKind;
    fn emit_extractor(
        &self,
        param: &Param,
        param_name: &proc_macro2::Ident,
        param_type: &proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream;
}

/// Central registry. Owned by `RustGenerator` (or built ad-hoc). Lookup is
/// `O(1)` via `HashMap` keyed on [`DecoratorKind`].
pub(crate) struct DecoratorRegistry {
    class: HashMap<DecoratorKind, Box<dyn ClassDecoratorHandler>>,
    method: HashMap<DecoratorKind, Box<dyn MethodDecoratorHandler>>,
    /// Param-level handler storage, populated in PR #3.
    #[allow(dead_code)]
    param: HashMap<DecoratorKind, Box<dyn ParamDecoratorHandler>>,
}

impl DecoratorRegistry {
    pub(crate) fn new() -> Self {
        Self {
            class: HashMap::new(),
            method: HashMap::new(),
            param: HashMap::new(),
        }
    }

    pub(crate) fn register_class(&mut self, handler: Box<dyn ClassDecoratorHandler>) {
        self.class.insert(handler.kind(), handler);
    }

    pub(crate) fn register_method(&mut self, handler: Box<dyn MethodDecoratorHandler>) {
        self.method.insert(handler.kind(), handler);
    }

    #[allow(dead_code)]
    pub(crate) fn register_param(&mut self, handler: Box<dyn ParamDecoratorHandler>) {
        self.param.insert(handler.kind(), handler);
    }

    pub(crate) fn class_handler(&self, kind: DecoratorKind) -> Option<&dyn ClassDecoratorHandler> {
        self.class.get(&kind).map(std::convert::AsRef::as_ref)
    }

    pub(crate) fn method_handler(
        &self,
        kind: DecoratorKind,
    ) -> Option<&dyn MethodDecoratorHandler> {
        self.method.get(&kind).map(std::convert::AsRef::as_ref)
    }

    /// Iterates the class decorators of `n`, classifies each by name,
    /// and invokes the corresponding handler. Unknown decorators are
    /// silently ignored — generic translation responsibility lies elsewhere.
    pub(crate) fn apply_class_decorators(&self, n: &ClassDecl, ctx: &mut ClassDecoratorContext) {
        for decorator in &n.class.decorators {
            let Some((kind, call)) = classify_decorator(decorator) else {
                continue;
            };
            if let Some(handler) = self.class_handler(kind) {
                handler.apply(n, call, ctx);
            }
        }
    }

    /// Iterates the decorators of a class method and invokes the matching
    /// method-level handlers. Unknown decorators are silently ignored.
    pub(crate) fn apply_method_decorators(
        &self,
        method: &ClassMethod,
        ctx: &mut MethodDecoratorContext,
    ) {
        for decorator in &method.function.decorators {
            let Some((kind, call)) = classify_decorator(decorator) else {
                continue;
            };
            if let Some(handler) = self.method_handler(kind) {
                handler.apply(method, call, ctx);
            }
        }
    }
}

/// Classifies a decorator AST node into its [`DecoratorKind`] and returns the
/// underlying [`CallExpr`] for handler use. Returns `None` for non-call
/// decorators or unknown names — the caller is expected to skip those.
fn classify_decorator(decorator: &swc_ecma_ast::Decorator) -> Option<(DecoratorKind, &CallExpr)> {
    let swc_ecma_ast::Expr::Call(call) = &*decorator.expr else {
        return None;
    };
    let swc_ecma_ast::Callee::Expr(callee_expr) = &call.callee else {
        return None;
    };
    let swc_ecma_ast::Expr::Ident(ident) = &**callee_expr else {
        return None;
    };
    DecoratorKind::from_name(ident.sym.as_ref()).map(|kind| (kind, call))
}

/// The default registry used by the transpiler. Adding a new built-in
/// decorator means adding one `register_*` line here.
pub(crate) fn default_registry() -> DecoratorRegistry {
    let mut registry = DecoratorRegistry::new();
    // Class-level
    registry.register_class(Box::new(controller::ControllerHandler));
    registry.register_class(Box::new(use_guards::UseGuardsHandler));
    // Method-level — one `HttpMethodHandler` instance per HTTP verb,
    // plus one `HttpCodeHandler` for `@HttpCode(...)`.
    for kind in [
        DecoratorKind::HttpGet,
        DecoratorKind::HttpPost,
        DecoratorKind::HttpPut,
        DecoratorKind::HttpDelete,
        DecoratorKind::HttpPatch,
    ] {
        registry.register_method(Box::new(http_method::HttpMethodHandler::new(kind)));
    }
    registry.register_method(Box::new(http_code::HttpCodeHandler));
    registry
}

/// Process-wide singleton of [`default_registry`]. Built once on first access;
/// subsequent calls are a pointer load.
pub(crate) fn shared_registry() -> &'static DecoratorRegistry {
    static REGISTRY: OnceLock<DecoratorRegistry> = OnceLock::new();
    REGISTRY.get_or_init(default_registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_class_handlers() {
        let r = default_registry();
        assert!(r.class_handler(DecoratorKind::Controller).is_some());
        assert!(r.class_handler(DecoratorKind::UseGuards).is_some());
        // Class lookup must not return method-/param-level handlers.
        assert!(r.class_handler(DecoratorKind::HttpGet).is_none());
    }

    #[test]
    fn default_registry_has_method_handlers() {
        let r = default_registry();
        for kind in [
            DecoratorKind::HttpGet,
            DecoratorKind::HttpPost,
            DecoratorKind::HttpPut,
            DecoratorKind::HttpDelete,
            DecoratorKind::HttpPatch,
            DecoratorKind::HttpCode,
        ] {
            assert!(r.method_handler(kind).is_some(), "{kind:?} not registered");
        }
        // Method lookup must not return class-level handlers.
        assert!(r.method_handler(DecoratorKind::Controller).is_none());
    }
}
