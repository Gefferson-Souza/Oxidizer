//! `@UseGuards(Guard1, Guard2)` handler.

use swc_ecma_ast::{CallExpr, ClassDecl, Expr};
use tyrus_decorator_kinds::DecoratorKind;

use super::{ClassContext, ClassDecoratorHandler};

pub(crate) struct UseGuardsHandler;

impl ClassDecoratorHandler for UseGuardsHandler {
    fn kind(&self) -> DecoratorKind {
        DecoratorKind::UseGuards
    }

    fn apply(&self, _class: &ClassDecl, call: &CallExpr, ctx: &mut ClassContext) {
        for arg in &call.args {
            if let Expr::Ident(ident) = &*arg.expr {
                ctx.guard_names.push(ident.sym.to_string());
            }
        }
    }
}
