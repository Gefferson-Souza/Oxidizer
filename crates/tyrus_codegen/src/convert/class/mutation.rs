use swc_ecma_ast::{AssignTarget, Expr, ExprStmt, Stmt};

use crate::convert::interface::RustGenerator;

impl RustGenerator {
    /// Checks if a method body contains assignments to `this.field` or
    /// mutating method calls on `this.field` (e.g., `this.history.push(...)`).
    pub(crate) fn body_mutates_self(stmts: &[Stmt]) -> bool {
        stmts.iter().any(Self::stmt_mutates_self)
    }

    fn stmt_mutates_self(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(ExprStmt { expr, .. }) => Self::expr_mutates_self(expr),
            Stmt::If(if_stmt) => {
                Self::stmt_mutates_self(&if_stmt.cons)
                    || if_stmt
                        .alt
                        .as_ref()
                        .is_some_and(|alt| Self::stmt_mutates_self(alt))
            }
            Stmt::Block(block) => Self::body_mutates_self(&block.stmts),
            Stmt::For(for_stmt) => Self::stmt_mutates_self(&for_stmt.body),
            Stmt::ForIn(for_in) => Self::stmt_mutates_self(&for_in.body),
            Stmt::ForOf(for_of) => Self::stmt_mutates_self(&for_of.body),
            Stmt::While(while_stmt) => Self::stmt_mutates_self(&while_stmt.body),
            _ => false,
        }
    }

    fn expr_mutates_self(expr: &Expr) -> bool {
        match expr {
            // this.field = ...
            Expr::Assign(assign) => {
                if let AssignTarget::Simple(simple) = &assign.left {
                    if let Some(member) = simple.as_member() {
                        if member.obj.is_this() {
                            return true;
                        }
                    }
                }
                false
            }
            // this.field.push(...) or similar mutating calls on this.field
            Expr::Call(call) => {
                if let swc_ecma_ast::Callee::Expr(callee_expr) = &call.callee {
                    if let Expr::Member(member) = &**callee_expr {
                        if let Expr::Member(inner) = &*member.obj {
                            if inner.obj.is_this() {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }
}
