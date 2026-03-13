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
        "push" => None, // Falls through to generic conversion: arr.push(x) works in Rust
        "map" => {
            // arr.map(x => x+1) -> arr.iter().map(|x| x+1).collect::<Vec<_>>()
            // This requires context of the closure.
            None
        }
        "filter" => {
            // arr.filter(x => x > 1)
            None
        }
        "find" => {
            if args.len() == 1 {
                let predicate = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { #obj_tokens.iter().find(|item| (#predicate)(**item)).cloned() })
            } else {
                None
            }
        }
        "join" => {
            if args.len() == 1 {
                let separator = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { #obj_tokens.join(&#separator) })
            } else {
                None
            }
        }
        "includes" => {
            if let Some(arg) = args.first() {
                let val = gen.convert_expr_or_spread(arg);
                Some(quote! { #obj_tokens.contains(&#val) })
            } else {
                None
            }
        }
        "indexOf" => {
            if args.len() == 1 {
                let val = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #obj_tokens.iter().position(|x| x == &#val).map(|i| i as f64).unwrap_or(-1.0)
                })
            } else {
                None
            }
        }
        "slice" => match args.len() {
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
        },
        "concat" => {
            if args.len() == 1 {
                let other = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #obj_tokens.iter().chain(#other.iter()).cloned().collect::<Vec<_>>()
                })
            } else {
                None
            }
        }
        "reverse" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.reverse() })
            } else {
                None
            }
        }
        "pop" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.pop() })
            } else {
                None
            }
        }
        "sort" => {
            if args.is_empty() {
                Some(quote! {
                    {
                        #obj_tokens.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    }
                })
            } else {
                None
            }
        }
        "shift" => {
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
        "flat" => {
            if args.is_empty() {
                Some(quote! {
                    #obj_tokens.iter().flatten().cloned().collect::<Vec<_>>()
                })
            } else {
                None
            }
        }
        "flatMap" => {
            if args.len() == 1 {
                let callback = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #obj_tokens.iter().flat_map(|x| (#callback)(x.clone())).collect::<Vec<_>>()
                })
            } else {
                None
            }
        }
        _ => None,
    }
}
