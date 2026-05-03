//! `@Get` / `@Post` / `@Put` / `@Delete` / `@Patch` handlers.
//!
//! A single struct parameterized by [`DecoratorKind`] backs all five HTTP
//! verb decorators. The registry stores one instance per verb in its method
//! handler map, so dispatch is still O(1) by kind.

use swc_ecma_ast::{CallExpr, ClassMethod, Expr, Lit};
use tyrus_decorator_kinds::DecoratorKind;

use super::{MethodDecoratorContext, MethodDecoratorHandler};

pub(crate) struct HttpMethodHandler {
    kind: DecoratorKind,
}

impl HttpMethodHandler {
    /// Creates a handler for the given HTTP verb decorator. The kind must be
    /// one of `HttpGet`, `HttpPost`, `HttpPut`, `HttpDelete`, `HttpPatch`.
    pub(crate) fn new(kind: DecoratorKind) -> Self {
        debug_assert!(
            kind.is_http_method(),
            "HttpMethodHandler instantiated with non-HTTP-method kind {kind:?}"
        );
        Self { kind }
    }
}

impl MethodDecoratorHandler for HttpMethodHandler {
    fn kind(&self) -> DecoratorKind {
        self.kind
    }

    fn apply(&self, _method: &ClassMethod, call: &CallExpr, ctx: &mut MethodDecoratorContext) {
        ctx.http_method = Some(self.kind);
        if let Some(arg) = call.args.first() {
            if let Expr::Lit(Lit::Str(s)) = &*arg.expr {
                ctx.route_path = s.value.as_str().unwrap_or_default().to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_carries_kind() {
        for kind in [
            DecoratorKind::HttpGet,
            DecoratorKind::HttpPost,
            DecoratorKind::HttpPut,
            DecoratorKind::HttpDelete,
            DecoratorKind::HttpPatch,
        ] {
            let h = HttpMethodHandler::new(kind);
            assert_eq!(h.kind(), kind);
            assert!(kind.is_http_method());
            assert!(kind.axum_routing_fn_name().is_some());
        }
    }
}
