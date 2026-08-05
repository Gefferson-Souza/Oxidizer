use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::{Expr, ExprOrSpread};

use super::super::convert::interface::RustGenerator;

/// Handle Map method calls (`HashMap` equivalents)
pub(crate) fn handle_map(
    gen: &RustGenerator,
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    let obj_tokens = gen.convert_expr(obj);
    match method {
        "set" => handle_map_set(gen, &obj_tokens, args),
        "get" => handle_map_get(gen, &obj_tokens, args),
        "has" => handle_map_has(gen, &obj_tokens, args),
        "delete" => handle_map_delete(gen, &obj_tokens, args),
        "clear" => Some(quote! { #obj_tokens.clear() }),
        "forEach" => handle_map_for_each(gen, &obj_tokens, args),
        _ => None,
    }
}

/// Handle Set method calls (`HashSet` equivalents)
pub(crate) fn handle_set(
    gen: &RustGenerator,
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    let obj_tokens = gen.convert_expr(obj);
    match method {
        "add" => handle_set_add(gen, &obj_tokens, args),
        "has" => handle_set_has(gen, &obj_tokens, args),
        "delete" => handle_set_delete(gen, &obj_tokens, args),
        "clear" => Some(quote! { #obj_tokens.clear() }),
        "forEach" => handle_set_for_each(gen, &obj_tokens, args),
        _ => None,
    }
}

// ── Map methods ──

fn handle_map_set(
    gen: &RustGenerator,
    obj: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let [key, value, ..] = args {
        let key = gen.convert_expr(&key.expr);
        let value = gen.convert_expr(&value.expr);
        Some(quote! { #obj.insert(#key, #value) })
    } else {
        None
    }
}

fn handle_map_get(
    gen: &RustGenerator,
    obj: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let Some(first) = args.first() {
        let key = gen.convert_expr(&first.expr);
        Some(quote! { #obj.get(&#key).cloned() })
    } else {
        None
    }
}

fn handle_map_has(
    gen: &RustGenerator,
    obj: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let Some(first) = args.first() {
        let key = gen.convert_expr(&first.expr);
        Some(quote! { #obj.contains_key(&#key) })
    } else {
        None
    }
}

fn handle_map_delete(
    gen: &RustGenerator,
    obj: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let Some(first) = args.first() {
        let key = gen.convert_expr(&first.expr);
        Some(quote! { #obj.remove(&#key) })
    } else {
        None
    }
}

fn handle_map_for_each(
    gen: &RustGenerator,
    obj: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let Some(first) = args.first() {
        let callback = gen.convert_expr(&first.expr);
        Some(quote! { #obj.iter().for_each(|(k, v)| (#callback)(v.clone(), k.clone())) })
    } else {
        None
    }
}

// ── Set methods ──

fn handle_set_add(
    gen: &RustGenerator,
    obj: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let Some(first) = args.first() {
        let value = gen.convert_expr(&first.expr);
        Some(quote! { #obj.insert(#value) })
    } else {
        None
    }
}

fn handle_set_has(
    gen: &RustGenerator,
    obj: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let Some(first) = args.first() {
        let value = gen.convert_expr(&first.expr);
        Some(quote! { #obj.contains(&#value) })
    } else {
        None
    }
}

fn handle_set_delete(
    gen: &RustGenerator,
    obj: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let Some(first) = args.first() {
        let value = gen.convert_expr(&first.expr);
        Some(quote! { #obj.remove(&#value) })
    } else {
        None
    }
}

fn handle_set_for_each(
    gen: &RustGenerator,
    obj: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let Some(first) = args.first() {
        let callback = gen.convert_expr(&first.expr);
        Some(quote! { #obj.iter().for_each(|v| (#callback)(v.clone())) })
    } else {
        None
    }
}
