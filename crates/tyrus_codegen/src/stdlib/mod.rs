use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::{Callee, Expr, ExprOrSpread};

use crate::convert::interface::RustGenerator;

pub(crate) mod array;
pub(crate) mod console;
pub(crate) mod json;
pub(crate) mod map_set;
pub(crate) mod math;
pub(crate) mod object;
pub(crate) mod string;

/// Main dispatcher for stdlib method calls
pub(crate) fn try_handle_stdlib_call(
    gen: &RustGenerator,
    callee: &Callee,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    // Try to handle as stdlib call
    if let Callee::Expr(expr) = callee {
        if let Expr::Member(member) = &**expr {
            if let Expr::Ident(obj) = &*member.obj {
                match obj.sym.as_ref() {
                    "console" => {
                        if let Some(prop) = member.prop.as_ident() {
                            return console::handle(gen, prop.sym.as_ref(), args);
                        }
                    }
                    "Math" => {
                        if let Some(prop) = member.prop.as_ident() {
                            return math::handle(gen, prop.sym.as_ref(), args);
                        }
                    }
                    "JSON" => {
                        if let Some(prop) = member.prop.as_ident() {
                            return json::handle(gen, prop.sym.as_ref(), args);
                        }
                    }
                    "Object" => {
                        if let Some(prop) = member.prop.as_ident() {
                            return object::handle(gen, prop.sym.as_ref(), args);
                        }
                    }
                    "Date" => {
                        if let Some(prop) = member.prop.as_ident() {
                            if prop.sym.as_ref() == "now" {
                                return Some(quote! {
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis() as f64
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

/// Try to handle method call on an expression (e.g., str.includes())
pub(crate) fn try_handle_method_call(
    gen: &RustGenerator,
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    // Check if object is a Map or Set variable first
    if let Expr::Ident(ident) = obj {
        let snake = crate::convert::helpers::to_snake_case(&ident.sym);
        if gen.map_vars.borrow().contains(&snake) {
            if let Some(res) = map_set::handle_map(gen, obj, method, args) {
                return Some(res);
            }
        }
        if gen.set_vars.borrow().contains(&snake) {
            if let Some(res) = map_set::handle_set(gen, obj, method, args) {
                return Some(res);
            }
        }
    }

    // For methods that exist on both String and Array (slice, indexOf, includes),
    // use a heuristic: check if the object looks like a string expression.
    let mut is_string = crate::convert::helpers::is_string_expr(obj);

    // Also check if the object is a variable declared with `: string` type annotation
    if !is_string {
        if let Expr::Ident(ident) = obj {
            let snake = crate::convert::helpers::to_snake_case(&ident.sym);
            is_string = gen.string_vars.borrow().contains(&snake);
        }
    }

    if is_string {
        if let Some(res) = string::handle(gen, obj, method, args) {
            return Some(res);
        }
        if let Some(res) = array::handle(gen, obj, method, args) {
            return Some(res);
        }
    } else {
        if let Some(res) = array::handle(gen, obj, method, args) {
            return Some(res);
        }
        if let Some(res) = string::handle(gen, obj, method, args) {
            return Some(res);
        }
    }

    None
}
