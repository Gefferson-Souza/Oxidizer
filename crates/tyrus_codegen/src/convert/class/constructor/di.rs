//! Build the secondary `new_di(...)` constructor for DI-friendly class
//! instantiation. Walks the same `Constructor::params` shape as `params.rs`
//! but emits a constructor that:
//!
//!   - takes Arc-wrapped dependency parameters,
//!   - assigns them directly to fields (not initializer-wrapped),
//!   - fills uninitialized fields with `Default::default()` (Mutex-wrapped
//!     for service/controller state),
//!   - appends `_marker: PhantomData` for generic classes.

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{Constructor, Pat};

use crate::convert::helpers::to_snake_case;
use crate::convert::type_mapper::map_ts_type;

use super::{is_dependency_type, ConstructorCtx};

pub(super) fn build_di_constructor(
    constructor: &Constructor,
    ctx: &ConstructorCtx<'_>,
) -> TokenStream {
    let mut di_params = Vec::new();
    let mut di_field_inits = Vec::new();
    let mut di_initialized = HashSet::new();

    for param in &constructor.params {
        match param {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) => {
                build_di_ts_param(
                    prop,
                    ctx,
                    &mut di_params,
                    &mut di_field_inits,
                    &mut di_initialized,
                );
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(pat_param) => {
                build_di_plain_param(
                    pat_param,
                    ctx,
                    &mut di_params,
                    &mut di_field_inits,
                    &mut di_initialized,
                );
            }
        }
    }

    if ctx.has_generics {
        di_field_inits.push(quote! { _marker: std::marker::PhantomData });
    }

    fill_di_defaults(
        ctx.class_fields,
        ctx.is_service_or_controller,
        &di_initialized,
        &mut di_field_inits,
    );

    quote! {
        pub fn new_di(#(#di_params),*) -> Self {
            Self {
                #(#di_field_inits),*
            }
        }
    }
}

fn build_di_ts_param(
    prop: &swc_ecma_ast::TsParamProp,
    ctx: &ConstructorCtx<'_>,
    di_params: &mut Vec<TokenStream>,
    di_field_inits: &mut Vec<TokenStream>,
    di_initialized: &mut HashSet<String>,
) {
    let swc_ecma_ast::TsParamPropParam::Ident(ident) = &prop.param else {
        return;
    };

    let param_name_str = ident.sym.to_string();
    let param_name = format_ident!("{}", to_snake_case(&param_name_str));
    let type_ann = ident.type_ann.as_ref();
    let mut param_type = map_ts_type(type_ann);

    let is_dep = ctx.is_service_or_controller
        && is_dependency_type(
            type_ann.map(std::convert::AsRef::as_ref),
            ctx.generic_params,
        );
    if is_dep {
        param_type = quote! { std::sync::Arc<#param_type> };
    }

    di_params.push(quote! { #param_name: #param_type });
    di_field_inits.push(quote! { #param_name: #param_name });
    di_initialized.insert(param_name_str);
}

fn build_di_plain_param(
    pat_param: &swc_ecma_ast::Param,
    ctx: &ConstructorCtx<'_>,
    di_params: &mut Vec<TokenStream>,
    di_field_inits: &mut Vec<TokenStream>,
    di_initialized: &mut HashSet<String>,
) {
    let Pat::Ident(ident) = &pat_param.pat else {
        return;
    };

    let param_name_str = ident.sym.to_string();
    let param_name = format_ident!("{}", &param_name_str);
    let param_type = map_ts_type(ident.type_ann.as_ref());

    let is_dep = ctx.is_service_or_controller
        && is_dependency_type(ident.type_ann.as_deref(), ctx.generic_params);
    if !is_dep {
        di_params.push(quote! { #param_name: #param_type });
        if ctx.class_fields.iter().any(|(n, _)| n == &param_name_str) {
            di_field_inits.push(quote! { #param_name: #param_name });
            di_initialized.insert(param_name_str);
        }
        return;
    }

    let arc_type = quote! { std::sync::Arc<#param_type> };
    di_params.push(quote! { #param_name: #arc_type });

    if ctx.class_fields.iter().any(|(n, _)| n == &param_name_str) {
        di_field_inits.push(quote! { #param_name: #param_name });
        di_initialized.insert(param_name_str);
    }
}

fn fill_di_defaults(
    class_fields: &[(String, bool)],
    is_service_or_controller: bool,
    di_initialized: &HashSet<String>,
    di_field_inits: &mut Vec<TokenStream>,
) {
    for (name, _) in class_fields {
        if !di_initialized.contains(name) {
            let field_name = format_ident!("{}", to_snake_case(name));
            if is_service_or_controller {
                di_field_inits.push(
                    quote! { #field_name: std::sync::Arc::new(std::sync::Mutex::new(Default::default())) },
                );
            } else {
                di_field_inits.push(quote! { #field_name: Default::default() });
            }
        }
    }
}
