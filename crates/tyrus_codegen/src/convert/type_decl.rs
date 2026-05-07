//! TypeScript type-alias and enum declaration → Rust translation.
//!
//! Two entry points used by `interface.rs`'s `Visit` impl:
//!   - `convert_type_alias` — handles `type X = "a" | "b"` (string union → enum)
//!     and the generic `type X = T` form
//!   - `convert_enum` — handles `enum X { ... }` (string-valued or numeric)
//!
//! Both return a single `TokenStream` ready to append to the generator's
//! output buffer. Splitting these out keeps `interface.rs` under the Rule 4
//! file ceiling and the two `visit_*` dispatchers small.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use swc_ecma_ast::{Lit, TsEnumDecl, TsEnumMember, TsEnumMemberId, TsTypeAliasDecl};

use super::helpers::to_pascal_case;
use super::type_mapper::map_ts_type;

pub(crate) fn convert_type_alias(decl: &TsTypeAliasDecl, is_exporting: bool) -> TokenStream {
    let alias_name = format_ident!("{}", decl.id.sym.to_string());

    if let Some(variants) = string_literal_union(&decl.type_ann) {
        return emit_string_union_enum(&alias_name, &variants, is_exporting);
    }

    emit_plain_type_alias(&alias_name, decl, is_exporting)
}

pub(crate) fn convert_enum(decl: &TsEnumDecl, is_exporting: bool) -> TokenStream {
    let enum_name = format_ident!("{}", decl.id.sym.to_string());
    if has_string_init(&decl.members) {
        emit_string_enum(&enum_name, decl, is_exporting)
    } else {
        emit_numeric_enum(&enum_name, decl, is_exporting)
    }
}

fn string_literal_union(type_ann: &swc_ecma_ast::TsType) -> Option<Vec<(String, Ident)>> {
    let swc_ecma_ast::TsType::TsUnionOrIntersectionType(
        swc_ecma_ast::TsUnionOrIntersectionType::TsUnionType(union),
    ) = type_ann
    else {
        return None;
    };
    if union.types.is_empty() {
        return None;
    }
    let mut variants = Vec::with_capacity(union.types.len());
    for t in &union.types {
        let swc_ecma_ast::TsType::TsLitType(lit) = &**t else {
            return None;
        };
        let swc_ecma_ast::TsLit::Str(s) = &lit.lit else {
            return None;
        };
        let value = s.value.as_str().unwrap_or("").to_string();
        let variant_ident = format_ident!("{}", to_pascal_case(&value));
        variants.push((value, variant_ident));
    }
    Some(variants)
}

fn emit_string_union_enum(
    alias_name: &Ident,
    valid_variants: &[(String, Ident)],
    is_exporting: bool,
) -> TokenStream {
    let variants = string_union_variant_tokens(valid_variants);
    let eq_arms_string = string_union_eq_arms(alias_name, valid_variants, false);
    let eq_arms_str = string_union_eq_arms(alias_name, valid_variants, true);
    let display_arms = string_union_display_arms(alias_name, valid_variants);
    let vis = visibility(is_exporting);

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
        #vis enum #alias_name {
            #(#variants),*
        }

        impl PartialEq<String> for #alias_name {
            fn eq(&self, other: &String) -> bool {
                match self {
                    #(#eq_arms_string),*
                }
            }
        }

        impl PartialEq<&str> for #alias_name {
            fn eq(&self, other: &&str) -> bool {
                match self {
                    #(#eq_arms_str),*
                }
            }
        }

        impl std::fmt::Display for #alias_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#display_arms),*
                }
            }
        }
    }
}

fn string_union_variant_tokens(valid_variants: &[(String, Ident)]) -> Vec<TokenStream> {
    valid_variants
        .iter()
        .enumerate()
        .map(|(i, (value, variant_ident))| {
            let default_attr = if i == 0 {
                quote! { #[default] }
            } else {
                quote! {}
            };
            quote! {
                #default_attr
                #[serde(rename = #value)]
                #variant_ident
            }
        })
        .collect()
}

fn string_union_eq_arms(
    alias_name: &Ident,
    valid_variants: &[(String, Ident)],
    deref_other: bool,
) -> Vec<TokenStream> {
    valid_variants
        .iter()
        .map(|(value, variant_ident)| {
            if deref_other {
                quote! { #alias_name::#variant_ident => *other == #value }
            } else {
                quote! { #alias_name::#variant_ident => other == #value }
            }
        })
        .collect()
}

fn string_union_display_arms(
    alias_name: &Ident,
    valid_variants: &[(String, Ident)],
) -> Vec<TokenStream> {
    valid_variants
        .iter()
        .map(|(value, variant_ident)| {
            quote! { #alias_name::#variant_ident => write!(f, #value) }
        })
        .collect()
}

fn emit_plain_type_alias(
    alias_name: &Ident,
    decl: &TsTypeAliasDecl,
    is_exporting: bool,
) -> TokenStream {
    let alias_type = map_ts_type(Some(&Box::new(swc_ecma_ast::TsTypeAnn {
        span: swc_common::DUMMY_SP,
        type_ann: decl.type_ann.clone(),
    })));
    let vis = visibility(is_exporting);
    quote! {
        #vis type #alias_name = #alias_type;
    }
}

fn has_string_init(members: &[TsEnumMember]) -> bool {
    members.iter().any(|m| {
        m.init
            .as_ref()
            .is_some_and(|init| matches!(init.as_ref(), swc_ecma_ast::Expr::Lit(Lit::Str(_))))
    })
}

fn member_ident_string(id: &TsEnumMemberId) -> String {
    match id {
        TsEnumMemberId::Ident(i) => i.sym.to_string(),
        TsEnumMemberId::Str(s) => s.value.as_str().unwrap_or("").to_string(),
    }
}

fn emit_string_enum(enum_name: &Ident, decl: &TsEnumDecl, is_exporting: bool) -> TokenStream {
    let variants: Vec<_> = decl.members.iter().map(string_enum_variant).collect();
    let vis = visibility(is_exporting);
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #vis enum #enum_name {
            #(#variants),*
        }
    }
}

fn string_enum_variant(m: &TsEnumMember) -> TokenStream {
    let variant_name_str = member_ident_string(&m.id);
    let variant_ident = format_ident!("{}", variant_name_str);
    let rename = m
        .init
        .as_ref()
        .and_then(|init| {
            if let swc_ecma_ast::Expr::Lit(Lit::Str(s)) = init.as_ref() {
                Some(s.value.as_str().unwrap_or("").to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| variant_name_str.clone());

    if rename == variant_name_str {
        quote! { #variant_ident }
    } else {
        quote! {
            #[serde(rename = #rename)]
            #variant_ident
        }
    }
}

fn emit_numeric_enum(enum_name: &Ident, decl: &TsEnumDecl, is_exporting: bool) -> TokenStream {
    let mut current_value: i64 = 0;
    let variants: Vec<_> = decl
        .members
        .iter()
        .map(|m| numeric_enum_variant(m, &mut current_value))
        .collect();
    let vis = visibility(is_exporting);
    let enum_def = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
        #[repr(i32)]
        #vis enum #enum_name {
            #[default]
            #(#variants),*
        }
    };
    let display_impl = quote! {
        impl std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", *self as i32)
            }
        }
    };
    quote! {
        #enum_def
        #display_impl
    }
}

fn numeric_enum_variant(m: &TsEnumMember, current_value: &mut i64) -> TokenStream {
    let variant_name_str = member_ident_string(&m.id);
    let variant_ident = format_ident!("{}", variant_name_str);
    if let Some(init) = &m.init {
        if let swc_ecma_ast::Expr::Lit(Lit::Num(num)) = init.as_ref() {
            *current_value = num.value as i64;
        }
    }
    let val = *current_value as i32;
    *current_value += 1;
    quote! { #variant_ident = #val }
}

fn visibility(is_exporting: bool) -> TokenStream {
    if is_exporting {
        quote! { pub }
    } else {
        quote! {}
    }
}
