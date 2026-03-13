use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

/// Handle string method calls
pub fn handle(
    gen: &RustGenerator,
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    let obj_tokens = gen.convert_expr(obj);
    match method {
        "includes" => {
            if let Some(arg) = args.first() {
                let val = gen.convert_expr_or_spread(arg);
                Some(quote! { #obj_tokens.contains(&#val as &str) })
            } else {
                None
            }
        }
        "replace" => {
            if args.len() == 2 {
                let pattern = gen.convert_expr_or_spread(&args[0]);
                let replacement = gen.convert_expr_or_spread(&args[1]);
                Some(quote! { #obj_tokens.replacen(&#pattern, &#replacement, 1) })
            } else {
                None
            }
        }
        "split" => {
            if let Some(arg) = args.first() {
                let delimiter = gen.convert_expr_or_spread(arg);
                Some(quote! { #obj_tokens.split(&#delimiter).collect::<Vec<_>>() })
            } else {
                None
            }
        }
        "toUpperCase" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.to_uppercase() })
            } else {
                None
            }
        }
        "toLowerCase" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.to_lowercase() })
            } else {
                None
            }
        }
        "trim" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.trim().to_string() })
            } else {
                None
            }
        }
        "startsWith" => {
            if let Some(arg) = args.first() {
                let val = gen.convert_expr_or_spread(arg);
                Some(quote! { #obj_tokens.starts_with(&#val as &str) })
            } else {
                None
            }
        }
        "endsWith" => {
            if let Some(arg) = args.first() {
                let val = gen.convert_expr_or_spread(arg);
                Some(quote! { #obj_tokens.ends_with(&#val as &str) })
            } else {
                None
            }
        }
        "toString" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.to_string() })
            } else {
                None
            }
        }
        "substring" | "slice" => match args.len() {
            1 => {
                let start = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { #obj_tokens[(#start as usize)..].to_string() })
            }
            2 => {
                let start = gen.convert_expr_or_spread(&args[0]);
                let end = gen.convert_expr_or_spread(&args[1]);
                Some(quote! { #obj_tokens[(#start as usize)..(#end as usize)].to_string() })
            }
            _ => None,
        },
        "charAt" => {
            if args.len() == 1 {
                let idx = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #obj_tokens.chars().nth(#idx as usize).map(|c| c.to_string()).unwrap_or_default()
                })
            } else {
                None
            }
        }
        "indexOf" => {
            if args.len() == 1 {
                let substr = gen.convert_expr_or_spread(&args[0]);
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
        "repeat" => {
            if args.len() == 1 {
                let n = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { #obj_tokens.repeat(#n as usize) })
            } else {
                None
            }
        }
        _ => None,
    }
}
