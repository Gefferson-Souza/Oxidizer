//! Shared helper functions for code generation.
//!
//! Contains utility functions used across multiple codegen modules:
//! - Case conversion (to_snake_case, to_pascal_case)
//! - Expression type detection (is_string_expr)

use swc_ecma_ast::*;

/// Convert camelCase or PascalCase to snake_case.
/// Handles consecutive uppercase by NOT inserting underscore between them.
/// Example: "getUserName" → "get_user_name", "HTTPRequest" → "httprequest"
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut was_upper = false;
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !was_upper {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            was_upper = true;
        } else {
            result.push(c);
            was_upper = false;
        }
    }
    result
}

/// Convert snake_case, camelCase, or kebab-case to PascalCase.
/// Handles both `_` and `-` delimiters.
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut next_upper = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            next_upper = true;
        } else if next_upper {
            result.push(c.to_ascii_uppercase());
            next_upper = false;
        } else {
            result.push(c);
        }
    }
    result
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case_simple() {
        assert_eq!(to_snake_case("getUserName"), "get_user_name");
    }

    #[test]
    fn test_to_snake_case_consecutive_upper() {
        assert_eq!(to_snake_case("HTTPRequest"), "httprequest");
    }

    #[test]
    fn test_to_snake_case_already_snake() {
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_to_pascal_case_from_snake() {
        assert_eq!(to_pascal_case("user_service"), "UserService");
    }

    #[test]
    fn test_to_pascal_case_from_kebab() {
        assert_eq!(to_pascal_case("my-component"), "MyComponent");
    }
}
