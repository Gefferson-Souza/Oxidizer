//! Case-conversion utilities shared across the workspace.
//!
//! These were previously duplicated between `tyrus_common::util` and
//! `tyrus_codegen::convert::helpers` with **divergent behavior** on consecutive
//! uppercase characters (e.g. `HTTPRequest` produced `h_t_t_p_request` here
//! versus `httprequest` in codegen). The codegen behavior wins because it
//! matches what the transpiler emits in production for all 235+ tests; this
//! file is now the single source of truth.

/// Converts `camelCase` or `PascalCase` to `snake_case`.
///
/// A `_` is inserted before each uppercase letter, **except** when the previous
/// character was also uppercase. That preserves acronyms as a single token
/// (`HTTPRequest` → `httprequest`, not `h_t_t_p_request`), which matches the
/// idiomatic Rust convention used by the generated handler names.
///
/// # Examples
///
/// ```
/// use tyrus_common::util::to_snake_case;
///
/// assert_eq!(to_snake_case("getUserName"), "get_user_name");
/// assert_eq!(to_snake_case("HTTPRequest"), "httprequest");
/// assert_eq!(to_snake_case("already_snake"), "already_snake");
/// assert_eq!(to_snake_case(""), "");
/// ```
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_upper = false;
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !prev_was_upper {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_was_upper = true;
        } else {
            result.push(ch);
            prev_was_upper = false;
        }
    }
    result
}

/// Converts `snake_case`, `camelCase`, or `kebab-case` to `PascalCase`.
/// Both `_` and `-` are recognized as word delimiters.
///
/// # Examples
///
/// ```
/// use tyrus_common::util::to_pascal_case;
///
/// assert_eq!(to_pascal_case("user_service"), "UserService");
/// assert_eq!(to_pascal_case("my-component"), "MyComponent");
/// assert_eq!(to_pascal_case("alreadyPascal"), "AlreadyPascal");
/// ```
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut next_upper = true;
    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            next_upper = true;
        } else if next_upper {
            result.push(ch.to_ascii_uppercase());
            next_upper = false;
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_simple_camel() {
        assert_eq!(to_snake_case("fetchData"), "fetch_data");
        assert_eq!(to_snake_case("getUserName"), "get_user_name");
    }

    #[test]
    fn snake_case_consecutive_uppercase_treated_as_acronym() {
        // The codegen behavior: HTTPRequest → httprequest (single token).
        // The previous tyrus_common behavior produced h_t_t_p_request, which
        // diverged from what `tyrus_codegen` emitted in production.
        assert_eq!(to_snake_case("HTTPRequest"), "httprequest");
        assert_eq!(to_snake_case("HTTP"), "http");
    }

    #[test]
    fn snake_case_already_snake_idempotent() {
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn snake_case_single_word() {
        assert_eq!(to_snake_case("simple"), "simple");
        assert_eq!(to_snake_case("Simple"), "simple");
    }

    #[test]
    fn snake_case_empty_string() {
        assert_eq!(to_snake_case(""), "");
    }

    #[test]
    fn pascal_case_from_snake() {
        assert_eq!(to_pascal_case("user_service"), "UserService");
    }

    #[test]
    fn pascal_case_from_kebab() {
        assert_eq!(to_pascal_case("my-component"), "MyComponent");
    }

    #[test]
    fn pascal_case_from_pascal_idempotent() {
        assert_eq!(to_pascal_case("AlreadyPascal"), "AlreadyPascal");
    }

    #[test]
    fn pascal_case_empty_string() {
        assert_eq!(to_pascal_case(""), "");
    }
}
