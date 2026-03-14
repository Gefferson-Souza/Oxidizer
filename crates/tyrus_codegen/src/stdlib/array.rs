use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

/// Handle array method calls
pub(crate) fn handle(
    gen: &RustGenerator,
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    let obj_tokens = gen.convert_expr(obj);
    match method {
        "push" => None,
        "map" => None,
        "filter" => None,
        "find" => handle_find(gen, &obj_tokens, args),
        "join" => handle_join(gen, &obj_tokens, args),
        "includes" => handle_includes(gen, &obj_tokens, args),
        "indexOf" => handle_index_of(gen, &obj_tokens, args),
        "slice" => handle_slice(gen, &obj_tokens, args),
        "concat" => handle_concat(gen, &obj_tokens, args),
        "reverse" => nullary_method(&obj_tokens, args, |o| quote! { #o.reverse() }),
        "pop" => nullary_method(&obj_tokens, args, |o| quote! { #o.pop() }),
        "sort" => handle_sort(&obj_tokens, args),
        "shift" => handle_shift(&obj_tokens, args),
        "flat" => handle_flat(&obj_tokens, args),
        "flatMap" => handle_flat_map(gen, &obj_tokens, args),
        _ => None,
    }
}

fn nullary_method(
    obj_tokens: &TokenStream,
    args: &[ExprOrSpread],
    emit: impl FnOnce(&TokenStream) -> TokenStream,
) -> Option<TokenStream> {
    if args.is_empty() {
        Some(emit(obj_tokens))
    } else {
        None
    }
}

fn handle_find(
    gen: &RustGenerator,
    obj_tokens: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if args.len() == 1 {
        let predicate = gen.convert_expr_or_spread(&args[0]);
        Some(quote! { #obj_tokens.iter().find(|item| (#predicate)(**item)).cloned() })
    } else {
        None
    }
}

fn handle_join(
    gen: &RustGenerator,
    obj_tokens: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if args.len() == 1 {
        let separator = gen.convert_expr_or_spread(&args[0]);
        Some(quote! { #obj_tokens.join(&#separator) })
    } else {
        None
    }
}

fn handle_includes(
    gen: &RustGenerator,
    obj_tokens: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if let Some(arg) = args.first() {
        let val = gen.convert_expr_or_spread(arg);
        Some(quote! { #obj_tokens.contains(&#val) })
    } else {
        None
    }
}

fn handle_index_of(
    gen: &RustGenerator,
    obj_tokens: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if args.len() == 1 {
        let val = gen.convert_expr_or_spread(&args[0]);
        Some(quote! {
            #obj_tokens.iter().position(|x| x == &#val).map(|i| i as f64).unwrap_or(-1.0)
        })
    } else {
        None
    }
}

fn handle_slice(
    gen: &RustGenerator,
    obj_tokens: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    match args.len() {
        1 => {
            let start = gen.convert_expr_or_spread(&args[0]);
            Some(quote! { #obj_tokens[(#start as usize)..].to_vec() })
        }
        2 => {
            let start = gen.convert_expr_or_spread(&args[0]);
            let end = gen.convert_expr_or_spread(&args[1]);
            Some(quote! { #obj_tokens[(#start as usize)..(#end as usize)].to_vec() })
        }
        _ => None,
    }
}

fn handle_concat(
    gen: &RustGenerator,
    obj_tokens: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if args.len() == 1 {
        let other = gen.convert_expr_or_spread(&args[0]);
        Some(quote! {
            #obj_tokens.iter().chain(#other.iter()).cloned().collect::<Vec<_>>()
        })
    } else {
        None
    }
}

fn handle_sort(obj_tokens: &TokenStream, args: &[ExprOrSpread]) -> Option<TokenStream> {
    if args.is_empty() {
        Some(quote! {
            {
                #obj_tokens.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            }
        })
    } else {
        None
    }
}

fn handle_shift(obj_tokens: &TokenStream, args: &[ExprOrSpread]) -> Option<TokenStream> {
    if args.is_empty() {
        Some(quote! {
            if #obj_tokens.is_empty() {
                Default::default()
            } else {
                #obj_tokens.remove(0)
            }
        })
    } else {
        None
    }
}

fn handle_flat(obj_tokens: &TokenStream, args: &[ExprOrSpread]) -> Option<TokenStream> {
    if args.is_empty() {
        Some(quote! {
            #obj_tokens.iter().flatten().cloned().collect::<Vec<_>>()
        })
    } else {
        None
    }
}

fn handle_flat_map(
    gen: &RustGenerator,
    obj_tokens: &TokenStream,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    if args.len() == 1 {
        let callback = gen.convert_expr_or_spread(&args[0]);
        Some(quote! {
            #obj_tokens.iter().flat_map(|x| (#callback)(x.clone())).collect::<Vec<_>>()
        })
    } else {
        None
    }
}
