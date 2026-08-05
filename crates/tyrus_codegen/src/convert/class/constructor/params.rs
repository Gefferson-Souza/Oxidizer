//! TypeScript constructor parameter extraction.
//!
//! Walks `constructor.params` and produces:
//!   - the lowered `param: Type` token streams,
//!   - the set of parameter names that are DI dependencies (Arc-wrapped),
//!   - the set of fields auto-initialized via TS parameter properties,
//!   - the matching `field: param` token streams for those auto-inits.

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{Constructor, Pat};

use crate::convert::helpers::to_snake_case;
use crate::convert::type_mapper::map_ts_type;

use super::is_dependency_type;

pub(super) struct ExtractedParams {
    pub params: Vec<TokenStream>,
    pub dependency_params: HashSet<String>,
    pub initialized_fields: HashSet<String>,
    pub field_inits: Vec<TokenStream>,
}

pub(super) fn extract_constructor_params(
    constructor: &Constructor,
    generic_params: &HashSet<String>,
    is_service_or_controller: bool,
) -> ExtractedParams {
    let mut result = ExtractedParams {
        params: Vec::new(),
        dependency_params: HashSet::new(),
        initialized_fields: HashSet::new(),
        field_inits: Vec::new(),
    };

    for param in &constructor.params {
        match param {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) => {
                extract_ts_param_prop(prop, generic_params, is_service_or_controller, &mut result);
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(pat_param) => {
                extract_plain_param(
                    pat_param,
                    generic_params,
                    is_service_or_controller,
                    &mut result,
                );
            }
        }
    }
    result
}

fn extract_ts_param_prop(
    prop: &swc_ecma_ast::TsParamProp,
    generic_params: &HashSet<String>,
    is_service_or_controller: bool,
    result: &mut ExtractedParams,
) {
    let swc_ecma_ast::TsParamPropParam::Ident(ident) = &prop.param else {
        return;
    };

    let param_name_str = ident.sym.to_string();
    let param_name = format_ident!("{}", to_snake_case(&param_name_str));
    let type_ann = ident.type_ann.as_ref();
    let mut param_type = map_ts_type(type_ann);

    let is_dep = is_service_or_controller
        && is_dependency_type(type_ann.map(std::convert::AsRef::as_ref), generic_params);
    if is_dep {
        param_type = quote! { std::sync::Arc<#param_type> };
        result.dependency_params.insert(param_name_str.clone());
    }

    result.params.push(quote! { #param_name: #param_type });
    result.field_inits.push(quote! { #param_name: #param_name });
    result.initialized_fields.insert(param_name_str);
}

fn extract_plain_param(
    pat_param: &swc_ecma_ast::Param,
    generic_params: &HashSet<String>,
    is_service_or_controller: bool,
    result: &mut ExtractedParams,
) {
    let Pat::Ident(ident) = &pat_param.pat else {
        return;
    };

    let param_name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
    let mut param_type = map_ts_type(ident.type_ann.as_ref());

    let is_dep =
        is_service_or_controller && is_dependency_type(ident.type_ann.as_deref(), generic_params);
    if is_dep {
        param_type = quote! { std::sync::Arc<#param_type> };
        result.dependency_params.insert(ident.sym.to_string());
    }

    result.params.push(quote! { #param_name: #param_type });
}
