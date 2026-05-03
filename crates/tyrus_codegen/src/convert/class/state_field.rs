//! State-field property emission for `@Injectable` / `@Controller` classes.
//!
//! Today every state field is wrapped in `Arc<Mutex<T>>` for interior
//! mutability. The wrapping decision lives in a single place
//! ([`wrap_state_field_type`]) so the upcoming atomic-state-fields work
//! (#143) can swap the wrapper based on the field's type without touching
//! the surrounding class-emission logic.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use swc_ecma_ast::ClassProp;

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;
use crate::convert::type_mapper::{is_optional_type, map_ts_type};

/// Convert a class field declaration into its emitted struct field token,
/// metadata, and wrapper-classification flags. Returns `None` if the field
/// has a non-ident key (computed property names are not supported).
///
/// Fields:
/// 0. emitted struct field (`pub <name>: <wrapped_type>`)
/// 1. raw field name (camelCase as written in TS)
/// 2. is the field "effectively optional" (Option-wrapped or `?`-suffixed)
/// 3. is the field a DI dependency (rather than mutable state)
/// 4. the raw inner type as a string — used by codegen to detect primitives
pub(super) fn convert_prop(
    gen: &RustGenerator,
    prop: &ClassProp,
    generic_params: &HashSet<String>,
    is_service_or_controller: bool,
) -> Option<(TokenStream, String, bool, bool, String)> {
    let field_name_str = prop.key.as_ident()?.sym.to_string();
    let field_name = format_ident!("{}", to_snake_case(&field_name_str));
    let inner_type = map_ts_type(prop.type_ann.as_ref());

    let raw_type_str = inner_type.to_string();

    let is_dependency = is_service_or_controller
        && super::constructor::is_dependency_type(prop.type_ann.as_deref(), generic_params);

    let mut field_type =
        wrap_state_field_type(&inner_type, is_dependency, is_service_or_controller);

    let is_optional_union = is_optional_type(prop.type_ann.as_deref());
    let is_effectively_optional = prop.is_optional || is_optional_union;

    if prop.is_optional {
        field_type = quote! { Option<#field_type> };
    }

    let _ = gen; // reserved for upcoming wrapper-kind selection (#143).
    Some((
        quote! { pub #field_name: #field_type },
        field_name_str,
        is_effectively_optional,
        is_dependency,
        raw_type_str,
    ))
}

/// Wrap a raw field type with the appropriate sharing/sync wrapper based on
/// the field's role. Centralised here so the atomic-state-fields refactor
/// (#143) can swap `Arc<Mutex<T>>` for `Arc<AtomicXxx>` per primitive type
/// without touching every emission site.
///
/// Roles:
/// * Dependency injection target → `Arc<T>` (shared, immutable from caller).
/// * Service/controller state → `Arc<Mutex<T>>` (interior mutability).
/// * DTO / entity field → raw type (no wrapper).
pub(super) fn wrap_state_field_type(
    inner: &TokenStream,
    is_dependency: bool,
    is_service_or_controller: bool,
) -> TokenStream {
    if is_dependency {
        quote! { std::sync::Arc<#inner> }
    } else if is_service_or_controller {
        quote! { std::sync::Arc<std::sync::Mutex<#inner>> }
    } else {
        inner.clone()
    }
}
