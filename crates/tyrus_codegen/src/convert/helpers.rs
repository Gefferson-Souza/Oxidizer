//! Shared helper functions for code generation.
//!
//! - Case conversion: re-exports `to_snake_case` and `to_pascal_case` from
//!   `tyrus_common::util` (single source of truth across the workspace)
//! - Expression type detection (`is_string_expr`)

use swc_ecma_ast::*;

pub use tyrus_common::util::{to_pascal_case, to_snake_case};

/// Heuristic to detect if an expression is likely a string type in TypeScript.
/// Used to choose `format!` over `+` for string concatenation.
pub(crate) fn is_string_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(Lit::Str(_)) => true,
        Expr::Tpl(_) => true,
        Expr::Call(call) => {
            if let Callee::Expr(callee) = &call.callee {
                if let Expr::Member(member) = &**callee {
                    if let Some(ident) = member.prop.as_ident() {
                        return matches!(
                            ident.sym.as_str(),
                            "toString"
                                | "toUpperCase"
                                | "toLowerCase"
                                | "trim"
                                | "replace"
                                | "join"
                                | "slice"
                                | "substring"
                                | "charAt"
                                | "concat"
                                | "padStart"
                                | "padEnd"
                                | "repeat"
                        );
                    }
                }
                // String::from(...) / String(...)
                if let Expr::Ident(ident) = &**callee {
                    return ident.sym.as_str() == "String";
                }
            }
            false
        }
        Expr::Bin(bin) if bin.op == BinaryOp::Add => {
            is_string_expr(&bin.left) || is_string_expr(&bin.right)
        }
        _ => false,
    }
}

// Case-conversion tests live in `tyrus_common::util` (the canonical
// implementation). `is_string_expr` heuristics for codegen would belong here
// but currently exercise their behavior via the equivalence-test suite.
