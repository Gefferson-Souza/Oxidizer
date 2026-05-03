//! `@HttpCode(201)` handler — sets the response status code on a method.

use swc_ecma_ast::{CallExpr, ClassMethod, Expr, Lit};
use tyrus_decorator_kinds::DecoratorKind;

use super::{MethodDecoratorContext, MethodDecoratorHandler};

pub(crate) struct HttpCodeHandler;

impl MethodDecoratorHandler for HttpCodeHandler {
    fn kind(&self) -> DecoratorKind {
        DecoratorKind::HttpCode
    }

    fn apply(&self, _method: &ClassMethod, call: &CallExpr, ctx: &mut MethodDecoratorContext) {
        if let Some(arg) = call.args.first() {
            if let Expr::Lit(Lit::Num(num)) = &*arg.expr {
                ctx.http_code = Some(num.value as u16);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_kind_is_http_code() {
        let h = HttpCodeHandler;
        assert_eq!(h.kind(), DecoratorKind::HttpCode);
        assert!(!DecoratorKind::HttpCode.is_http_method());
        assert!(DecoratorKind::HttpCode.axum_routing_fn_name().is_none());
    }
}
