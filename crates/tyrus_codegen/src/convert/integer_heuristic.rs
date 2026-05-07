//! Field-name heuristic that detects struct fields with integer-shape
//! semantics and emits the necessary serde attribute so JSON output
//! shows `{"id": 1}` instead of `{"id": 1.0}`.
//!
//! Rationale: TypeScript's single `number` type forces every field to
//! `f64` in the generated Rust struct. Real API consumers (front-end SPAs,
//! mobile apps, downstream services with strict JSON schemas) expect
//! integer-shaped fields like `id`, `userId`, or `count` to round-trip as
//! integers. Issue #130 documents the breakage.
//!
//! Strategy v1 (this module): keep the field as `f64` so internal
//! arithmetic and literal assignments compile unchanged, and attach
//! `#[serde(with = "crate::serde_helpers::f64_as_int")]` so the JSON
//! boundary serializes/deserializes as integer. This avoids the
//! "type mismatch" cascade that would follow if we flipped the field
//! to `i64` without rewriting every literal site.
//!
//! The helper module `serde_helpers::f64_as_int` is emitted by
//! `tyrus_orchestrator::scaffold::get_serde_helpers_code()`.
//!
//! The rule list is deliberately conservative: only field names whose
//! integer semantics are unambiguous are flagged.

use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::{TsKeywordTypeKind, TsType, TsTypeAnn};

use super::helpers::to_snake_case;

/// True when the field name implies integer-shape semantics.
pub(crate) fn is_integer_shaped_field(field_name: &str) -> bool {
    let snake = to_snake_case(field_name);

    if matches!(
        snake.as_str(),
        "id" | "index"
            | "size"
            | "length"
            | "count"
            | "timestamp"
            | "epoch"
            | "year"
            | "month"
            | "day"
            | "port"
            | "status_code"
            | "http_status"
    ) {
        return true;
    }

    snake.ends_with("_id")
        || snake.ends_with("_count")
        || snake.ends_with("_index")
        || snake.ends_with("_size")
        || snake.ends_with("_length")
        || snake.ends_with("_year")
        || snake.ends_with("_month")
        || snake.ends_with("_day")
        || snake.ends_with("_port")
        || snake.ends_with("_timestamp")
}

/// True when `type_ann` is exactly the `number` keyword (not a union,
/// not a generic, not a reference). The heuristic only applies when both
/// the name AND the bare-`number` shape match.
pub(crate) fn is_plain_number(type_ann: Option<&TsTypeAnn>) -> bool {
    let Some(ann) = type_ann else {
        return false;
    };
    let TsType::TsKeywordType(kw) = &*ann.type_ann else {
        return false;
    };
    kw.kind == TsKeywordTypeKind::TsNumberKeyword
}

/// Returns the serde attribute to attach to a struct field if the integer
/// heuristic applies. Empty `TokenStream` otherwise.
#[allow(clippy::borrowed_box)]
pub(crate) fn integer_serde_attr(
    type_ann: Option<&Box<TsTypeAnn>>,
    field_name: &str,
) -> TokenStream {
    if !is_integer_shaped_field(field_name) {
        return quote! {};
    }
    if !is_plain_number(type_ann.map(std::convert::AsRef::as_ref)) {
        return quote! {};
    }
    quote! { #[serde(with = "__tyrus_int_serde")] }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn detect_id_exact() {
        assert!(is_integer_shaped_field("id"));
    }

    #[test]
    fn detect_user_id_camel() {
        assert!(is_integer_shaped_field("userId"));
    }

    #[test]
    fn detect_user_id_snake() {
        assert!(is_integer_shaped_field("user_id"));
    }

    #[test]
    fn detect_view_count() {
        assert!(is_integer_shaped_field("viewCount"));
    }

    #[test]
    fn detect_status_code() {
        assert!(is_integer_shaped_field("statusCode"));
    }

    #[test]
    fn skip_non_integer_field_name() {
        assert!(!is_integer_shaped_field("name"));
        assert!(!is_integer_shaped_field("price"));
        assert!(!is_integer_shaped_field("ratio"));
        assert!(!is_integer_shaped_field("score"));
    }

    #[test]
    fn skip_id_inside_other_word() {
        assert!(!is_integer_shaped_field("guidance"));
    }
}
