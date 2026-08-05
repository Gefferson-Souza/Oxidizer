use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::{Expr, ExprOrSpread};

use super::super::convert::interface::RustGenerator;

/// Handle string method calls
pub(crate) fn handle(
    gen: &RustGenerator,
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    let obj_tokens = gen.convert_expr(obj);
    match method {
        "includes" => unary_str_method(gen, args, &obj_tokens, |v| {
            quote! { #obj_tokens.contains(&#v as &str) }
        }),
        "replace" => handle_replace(gen, args, &obj_tokens),
        "split" => unary_str_method(gen, args, &obj_tokens, |d| {
            quote! { #obj_tokens.split(&#d).collect::<Vec<_>>() }
        }),
        "toUpperCase" => nullary_method(args, || quote! { #obj_tokens.to_uppercase() }),
        "toLowerCase" => nullary_method(args, || quote! { #obj_tokens.to_lowercase() }),
        "trim" => nullary_method(args, || quote! { #obj_tokens.trim().to_string() }),
        "startsWith" => unary_str_method(gen, args, &obj_tokens, |v| {
            quote! { #obj_tokens.starts_with(&#v as &str) }
        }),
        "endsWith" => unary_str_method(gen, args, &obj_tokens, |v| {
            quote! { #obj_tokens.ends_with(&#v as &str) }
        }),
        "toString" => nullary_method(args, || quote! { #obj_tokens.to_string() }),
        "substring" | "slice" => handle_substring(gen, args, &obj_tokens),
        "charAt" => handle_char_at(gen, args, &obj_tokens),
        "indexOf" => handle_index_of(gen, args, &obj_tokens),
        "repeat" => handle_repeat(gen, args, &obj_tokens),
        "padStart" => pad_tokens(&obj_tokens, args, gen, true),
        "padEnd" => pad_tokens(&obj_tokens, args, gen, false),
        _ => None,
    }
}

fn nullary_method(
    args: &[ExprOrSpread],
    emit: impl FnOnce() -> TokenStream,
) -> Option<TokenStream> {
    if args.is_empty() {
        Some(emit())
    } else {
        None
    }
}

fn unary_str_method(
    gen: &RustGenerator,
    args: &[ExprOrSpread],
    _obj_tokens: &TokenStream,
    emit: impl FnOnce(TokenStream) -> TokenStream,
) -> Option<TokenStream> {
    args.first()
        .map(|arg| emit(gen.convert_expr_or_spread(arg)))
}

fn handle_replace(
    gen: &RustGenerator,
    args: &[ExprOrSpread],
    obj_tokens: &TokenStream,
) -> Option<TokenStream> {
    if let [pattern, replacement] = args {
        let pattern = gen.convert_expr_or_spread(pattern);
        let replacement = gen.convert_expr_or_spread(replacement);
        Some(quote! { #obj_tokens.replacen(&#pattern, &#replacement, 1) })
    } else {
        None
    }
}

fn handle_substring(
    gen: &RustGenerator,
    args: &[ExprOrSpread],
    obj_tokens: &TokenStream,
) -> Option<TokenStream> {
    match args {
        [start] => {
            let start = gen.convert_expr_or_spread(start);
            Some(quote! { #obj_tokens[(#start as usize)..].to_string() })
        }
        [start, end] => {
            let start = gen.convert_expr_or_spread(start);
            let end = gen.convert_expr_or_spread(end);
            Some(quote! { #obj_tokens[(#start as usize)..(#end as usize)].to_string() })
        }
        _ => None,
    }
}

fn handle_char_at(
    gen: &RustGenerator,
    args: &[ExprOrSpread],
    obj_tokens: &TokenStream,
) -> Option<TokenStream> {
    if let [arg] = args {
        let idx = gen.convert_expr_or_spread(arg);
        Some(quote! {
            #obj_tokens.chars().nth(#idx as usize).map(|c| c.to_string()).unwrap_or_default()
        })
    } else {
        None
    }
}

fn handle_index_of(
    gen: &RustGenerator,
    args: &[ExprOrSpread],
    obj_tokens: &TokenStream,
) -> Option<TokenStream> {
    if let [arg] = args {
        let substr = gen.convert_expr_or_spread(arg);
        Some(quote! {
            match #obj_tokens.find(&#substr as &str) {
                Some(i) => i as f64,
                None => -1.0,
            }
        })
    } else {
        None
    }
}

fn handle_repeat(
    gen: &RustGenerator,
    args: &[ExprOrSpread],
    obj_tokens: &TokenStream,
) -> Option<TokenStream> {
    if let [arg] = args {
        let n = gen.convert_expr_or_spread(arg);
        Some(quote! { #obj_tokens.repeat(#n as usize) })
    } else {
        None
    }
}

/// Generate padStart or padEnd tokens. Uses `chars().count()` for correct Unicode handling.
fn pad_tokens(
    obj_tokens: &TokenStream,
    args: &[ExprOrSpread],
    gen: &RustGenerator,
    is_start: bool,
) -> Option<TokenStream> {
    let (first, rest) = args.split_first()?;
    let target_len = gen.convert_expr_or_spread(first);
    match rest.first() {
        Some(fill_arg) => {
            let fill = gen.convert_expr_or_spread(fill_arg);
            Some(pad_with_fill(obj_tokens, &target_len, &fill, is_start))
        }
        None => Some(pad_with_space(obj_tokens, &target_len, is_start)),
    }
}

fn pad_with_fill(
    obj_tokens: &TokenStream,
    target_len: &TokenStream,
    fill: &TokenStream,
    is_start: bool,
) -> TokenStream {
    let fmt = if is_start {
        quote! { format!("{}{}", __pad, __s) }
    } else {
        quote! { format!("{}{}", __s, __pad) }
    };
    quote! {{
        let __s = #obj_tokens;
        let __target = #target_len as usize;
        let __fill: String = #fill.to_string();
        let __char_count = __s.chars().count();
        if __char_count >= __target {
            __s
        } else {
            let __pad_len = __target - __char_count;
            let __pad: String = __fill.chars().cycle().take(__pad_len).collect();
            #fmt
        }
    }}
}

fn pad_with_space(
    obj_tokens: &TokenStream,
    target_len: &TokenStream,
    is_start: bool,
) -> TokenStream {
    let fmt = if is_start {
        quote! { format!("{:>width$}", __s, width = __target) }
    } else {
        quote! { format!("{:<width$}", __s, width = __target) }
    };
    quote! {{
        let __s = #obj_tokens;
        let __target = #target_len as usize;
        if __s.chars().count() >= __target {
            __s
        } else {
            #fmt
        }
    }}
}
