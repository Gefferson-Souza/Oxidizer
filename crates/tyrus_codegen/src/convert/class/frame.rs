//! Working-state structs and pure helpers used by `process_class_decl`.
//!
//! Splitting these out keeps `class/mod.rs` under the Rule 4 file ceiling
//! while letting the dispatcher there stay small and read top-down.

use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use swc_ecma_ast::{ClassDecl, ClassMethod, Constructor};

use super::routing::{self, ControllerInfo};

/// Class-level metadata derived from `ClassDecl` before fields/methods are processed.
pub(super) struct ClassMeta {
    pub class_name: String,
    pub is_controller: bool,
    pub is_service_or_controller: bool,
    pub controller_info: ControllerInfo,
    pub generic_params: HashSet<String>,
    pub parent_class_name: Option<String>,
}

/// Members of the class categorized by kind.
pub(super) struct ClassMembers<'a> {
    pub methods: Vec<&'a ClassMethod>,
    pub getters: Vec<&'a ClassMethod>,
    pub setters: Vec<&'a ClassMethod>,
    pub constructor: Option<&'a Constructor>,
    pub static_method_names: HashSet<String>,
}

/// Aggregated field information collected from properties and constructor params.
pub(super) struct ClassFields {
    pub fields: Vec<TokenStream>,
    pub class_fields_meta: Vec<(String, bool)>,
    pub own_field_names: Vec<(String, TokenStream, bool)>,
    pub dependency_fields: HashSet<String>,
}

/// Token streams describing the class's generic parameters in three positions.
pub(super) struct ClassGenerics {
    pub struct_decl: TokenStream,
    pub impl_decl: TokenStream,
    pub use_tokens: TokenStream,
}

/// Aggregated working state for a single class transpilation pass.
pub(super) struct ClassFrame<'a> {
    pub decl: &'a ClassDecl,
    pub struct_name: Ident,
    pub meta: ClassMeta,
    pub members: ClassMembers<'a>,
    pub fields: ClassFields,
    pub generics: ClassGenerics,
}

pub(super) fn extract_class_meta(n: &ClassDecl) -> ClassMeta {
    let class_name = n.ident.sym.to_string();
    let controller_info = routing::extract_controller_info(n);
    let is_controller = controller_info.is_controller;
    let is_service_or_controller = is_controller || class_name.ends_with("Service");

    let mut generic_params = HashSet::new();
    if let Some(type_params) = &n.class.type_params {
        for param in &type_params.params {
            generic_params.insert(param.name.sym.to_string());
        }
    }

    let parent_class_name = n.class.super_class.as_ref().and_then(|expr| {
        if let swc_ecma_ast::Expr::Ident(ident) = &**expr {
            Some(ident.sym.to_string())
        } else {
            None
        }
    });

    ClassMeta {
        class_name,
        is_controller,
        is_service_or_controller,
        controller_info,
        generic_params,
        parent_class_name,
    }
}

pub(super) fn compute_class_generics(
    n: &ClassDecl,
    fields: &mut Vec<TokenStream>,
) -> ClassGenerics {
    let Some(type_params) = &n.class.type_params else {
        return ClassGenerics {
            struct_decl: quote! {},
            impl_decl: quote! {},
            use_tokens: quote! {},
        };
    };

    let params_struct: Vec<_> = type_params
        .params
        .iter()
        .map(|p| format_ident!("{}", p.name.sym.to_string()))
        .collect();

    let params_impl: Vec<_> = type_params
        .params
        .iter()
        .map(|p| {
            let ident = format_ident!("{}", p.name.sym.to_string());
            quote! { #ident: serde::de::DeserializeOwned + serde::Serialize + Clone + Default + std::fmt::Debug }
        })
        .collect();

    let params_use = params_struct.clone();

    if !params_use.is_empty() {
        let phantom_type = if params_use.len() == 1 {
            let p = &params_use[0];
            quote! { #p }
        } else {
            quote! { (#(#params_use),*) }
        };
        fields.push(quote! {
            #[serde(skip)]
            pub _marker: std::marker::PhantomData<#phantom_type>
        });
    }

    ClassGenerics {
        struct_decl: quote! { <#(#params_struct),*> },
        impl_decl: quote! { <#(#params_impl),*> },
        use_tokens: quote! { <#(#params_use),*> },
    }
}

pub(super) fn struct_derives(is_service_or_controller: bool) -> Vec<TokenStream> {
    let mut derives = vec![quote! { Default }, quote! { Debug }, quote! { Clone }];
    if !is_service_or_controller {
        derives.push(quote! { PartialEq });
        derives.push(quote! { serde::Serialize });
        derives.push(quote! { serde::Deserialize });
    }
    derives
}
