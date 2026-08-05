use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::{TsType, TsTypeAnn};

/// Maps a TS keyword type (string, number, boolean, void) to its Rust equivalent.
fn map_keyword_type(kind: swc_ecma_ast::TsKeywordTypeKind) -> TokenStream {
    match kind {
        swc_ecma_ast::TsKeywordTypeKind::TsStringKeyword => quote! { String },
        swc_ecma_ast::TsKeywordTypeKind::TsNumberKeyword => quote! { f64 },
        swc_ecma_ast::TsKeywordTypeKind::TsBooleanKeyword => quote! { bool },
        swc_ecma_ast::TsKeywordTypeKind::TsVoidKeyword => quote! { () },
        _ => quote! { serde_json::Value },
    }
}

/// Maps a TS type reference (Date, Array<T>, Record<K,V>, user-defined) to Rust.
fn map_type_ref(type_ref: &swc_ecma_ast::TsTypeRef) -> TokenStream {
    let Some(ident) = type_ref.type_name.as_ident() else {
        return quote! { serde_json::Value };
    };
    let name = ident.sym.as_str();
    match name {
        "Date" => quote! { String },
        "Array" => map_array_type(type_ref.type_params.as_deref()),
        "Record" | "Map" => map_record_type(type_ref.type_params.as_deref()),
        "Set" => map_set_type(type_ref.type_params.as_deref()),
        _ => map_user_defined_type(name, type_ref.type_params.as_deref()),
    }
}

/// Extracts the first generic param as `Vec<T>`, defaulting to `Vec<Value>`.
fn map_array_type(type_params: Option<&swc_ecma_ast::TsTypeParamInstantiation>) -> TokenStream {
    if let Some(params) = type_params {
        if let Some(first) = params.params.first() {
            let inner = map_type_core(first);
            return quote! { Vec<#inner> };
        }
    }
    quote! { Vec<serde_json::Value> }
}

/// Maps Record<K,V> to `HashMap`<K,V>, defaulting to `HashMap`<String, Value>.
fn map_record_type(type_params: Option<&swc_ecma_ast::TsTypeParamInstantiation>) -> TokenStream {
    if let Some(params) = type_params {
        if let [key, value, ..] = params.params.as_slice() {
            let key = map_type_core(key);
            let value = map_type_core(value);
            return quote! { std::collections::HashMap<#key, #value> };
        }
    }
    quote! { std::collections::HashMap<String, serde_json::Value> }
}

/// Maps Set<T> to `HashSet`<T>, defaulting to `HashSet`<`serde_json::Value`>.
fn map_set_type(type_params: Option<&swc_ecma_ast::TsTypeParamInstantiation>) -> TokenStream {
    if let Some(params) = type_params {
        if let Some(first) = params.params.first() {
            let inner = map_type_core(first);
            return quote! { std::collections::HashSet<#inner> };
        }
    }
    quote! { std::collections::HashSet<serde_json::Value> }
}

/// Maps a user-defined type reference, preserving generic parameters.
fn map_user_defined_type(
    name: &str,
    type_params: Option<&swc_ecma_ast::TsTypeParamInstantiation>,
) -> TokenStream {
    let type_ident = proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
    if let Some(params) = type_params {
        let mapped: Vec<_> = params.params.iter().map(|p| map_type_core(p)).collect();
        quote! { #type_ident<#(#mapped),*> }
    } else {
        quote! { #type_ident }
    }
}

/// Maps a TS union type (T | undefined -> Option<T>), falling back to Value.
fn map_union_type(union_or_intersection: &swc_ecma_ast::TsUnionOrIntersectionType) -> TokenStream {
    let swc_ecma_ast::TsUnionOrIntersectionType::TsUnionType(union) = union_or_intersection else {
        return quote! { serde_json::Value };
    };

    let mut is_optional = false;
    let mut inner_type = None;

    for type_opt in &union.types {
        match &**type_opt {
            TsType::TsKeywordType(k)
                if k.kind == swc_ecma_ast::TsKeywordTypeKind::TsUndefinedKeyword
                    || k.kind == swc_ecma_ast::TsKeywordTypeKind::TsNullKeyword =>
            {
                is_optional = true;
            }
            _ => {
                if inner_type.is_none() {
                    inner_type = Some(map_type_core(type_opt));
                }
            }
        }
    }

    if is_optional {
        let inner = inner_type.unwrap_or_else(|| quote! { serde_json::Value });
        quote! { Option<#inner> }
    } else {
        quote! { serde_json::Value }
    }
}

/// Core type mapping dispatcher: routes a `TsType` to the appropriate handler.
fn map_type_core(ts_type: &TsType) -> TokenStream {
    match ts_type {
        TsType::TsKeywordType(k) => map_keyword_type(k.kind),
        TsType::TsArrayType(arr) => {
            let inner = map_type_core(&arr.elem_type);
            quote! { Vec<#inner> }
        }
        TsType::TsTypeRef(t) => map_type_ref(t),
        TsType::TsUnionOrIntersectionType(u) => map_union_type(u),
        _ => quote! { serde_json::Value },
    }
}

/// Maps TypeScript types to Rust types from an optional type annotation.
// Why allowed: signature mirrors SWC's typed AST shape (`Option<&Box<TsTypeAnn>>`).
// Relaxing to `Option<&TsTypeAnn>` would force every call site to unwrap the Box.
#[allow(clippy::borrowed_box)]
pub fn map_ts_type(type_ann: Option<&Box<TsTypeAnn>>) -> TokenStream {
    if let Some(type_ann) = type_ann {
        map_type_core(&type_ann.type_ann)
    } else {
        quote! { serde_json::Value }
    }
}

/// Unwraps Promise<T> to T for async function return types
// Why allowed: same SWC-shaped signature as map_ts_type above; relaxing forces
// Box-unwrapping at every call site.
#[allow(clippy::borrowed_box)]
pub fn unwrap_promise_type(type_ann: Option<&Box<TsTypeAnn>>) -> TokenStream {
    if let Some(type_ann) = type_ann {
        if let TsType::TsTypeRef(type_ref) = &*type_ann.type_ann {
            // Check if this is a Promise type
            if let Some(ident) = type_ref.type_name.as_ident() {
                if ident.sym == "Promise" {
                    // Extract the generic parameter T from Promise<T>
                    if let Some(type_params) = &type_ref.type_params {
                        if let Some(first_param) = type_params.params.first() {
                            // Recursively map the inner type
                            return map_type_core(first_param);
                        }
                    }
                }
            }
        }
    }
    // If not a Promise or no generic, fall back to regular mapping
    map_ts_type(type_ann)
}

pub fn is_optional_type(type_ann: Option<&TsTypeAnn>) -> bool {
    if let Some(type_ann) = type_ann {
        if let TsType::TsUnionOrIntersectionType(
            swc_ecma_ast::TsUnionOrIntersectionType::TsUnionType(union),
        ) = &*type_ann.type_ann
        {
            for type_opt in &union.types {
                if let TsType::TsKeywordType(k) = &**type_opt {
                    if k.kind == swc_ecma_ast::TsKeywordTypeKind::TsUndefinedKeyword
                        || k.kind == swc_ecma_ast::TsKeywordTypeKind::TsNullKeyword
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn is_void_or_promise_void(type_ann: Option<&TsTypeAnn>) -> bool {
    if let Some(type_ann) = type_ann {
        match &*type_ann.type_ann {
            TsType::TsKeywordType(k) => k.kind == swc_ecma_ast::TsKeywordTypeKind::TsVoidKeyword,
            TsType::TsTypeRef(type_ref) => {
                // Check Promise<void>
                if let Some(ident) = type_ref.type_name.as_ident() {
                    if ident.sym == "Promise" {
                        if let Some(type_params) = &type_ref.type_params {
                            if let Some(first_param) = type_params.params.first() {
                                // Check if inner is void
                                if let TsType::TsKeywordType(k) = &**first_param {
                                    return k.kind
                                        == swc_ecma_ast::TsKeywordTypeKind::TsVoidKeyword;
                                }
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Maps a `TsType` directly to a Rust `TokenStream`.
pub fn map_inner_type(ts_type: &swc_ecma_ast::TsType) -> TokenStream {
    map_type_core(ts_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_common::DUMMY_SP;
    use swc_ecma_ast::{TsKeywordType, TsKeywordTypeKind};

    #[test]
    fn test_map_ts_type_string() {
        let ts_type = TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsStringKeyword,
        });
        let type_ann = Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann: Box::new(ts_type),
        });
        let result = map_ts_type(Some(&type_ann));
        assert_eq!(result.to_string(), "String");
    }

    #[test]
    fn test_map_ts_type_number() {
        let ts_type = TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsNumberKeyword,
        });
        let type_ann = Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann: Box::new(ts_type),
        });
        let result = map_ts_type(Some(&type_ann));
        assert_eq!(result.to_string(), "f64");
    }

    #[test]
    fn test_map_ts_type_boolean() {
        let ts_type = TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsBooleanKeyword,
        });
        let type_ann = Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann: Box::new(ts_type),
        });
        let result = map_ts_type(Some(&type_ann));
        assert_eq!(result.to_string(), "bool");
    }

    #[test]
    fn test_map_ts_type_none() {
        let result = map_ts_type(None);
        assert_eq!(result.to_string(), "serde_json :: Value");
    }
}
