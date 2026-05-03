//! `@Controller("/path")` handler.

use swc_ecma_ast::{CallExpr, ClassDecl, Expr, Lit};
use tyrus_decorator_kinds::DecoratorKind;

use super::{ClassDecoratorContext, ClassDecoratorHandler};

pub(crate) struct ControllerHandler;

impl ClassDecoratorHandler for ControllerHandler {
    fn kind(&self) -> DecoratorKind {
        DecoratorKind::Controller
    }

    fn apply(&self, _class: &ClassDecl, call: &CallExpr, ctx: &mut ClassDecoratorContext) {
        ctx.is_controller = true;
        if let Some(arg) = call.args.first() {
            if let Expr::Lit(Lit::Str(s)) = &*arg.expr {
                ctx.controller_path = s.value.as_str().unwrap_or_default().to_string();
            }
        }
    }
}
