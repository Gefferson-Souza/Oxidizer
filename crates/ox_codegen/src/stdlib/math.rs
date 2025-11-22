use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::func::convert_expr_or_spread;

/// Handle Math.* calls
pub fn handle(method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    match method {
        "max" => {
            if args.len() == 2 {
                let a = convert_expr_or_spread(&args[0]);
                let b = convert_expr_or_spread(&args[1]);
                Some(quote! { #a.max(#b) })
            } else {
                None
            }
        }
        "min" => {
            if args.len() == 2 {
                let a = convert_expr_or_spread(&args[0]);
                let b = convert_expr_or_spread(&args[1]);
                Some(quote! { #a.min(#b) })
            } else {
                None
            }
        }
        "round" => {
            if args.len() == 1 {
                let x = convert_expr_or_spread(&args[0]);
                Some(quote! { #x.round() })
            } else {
                None
            }
        }
        "floor" => {
            if args.len() == 1 {
                let x = convert_expr_or_spread(&args[0]);
                Some(quote! { #x.floor() })
            } else {
                None
            }
        }
        "ceil" => {
            if args.len() == 1 {
                let x = convert_expr_or_spread(&args[0]);
                Some(quote! { #x.ceil() })
            } else {
                None
            }
        }
        "abs" => {
            if args.len() == 1 {
                let x = convert_expr_or_spread(&args[0]);
                Some(quote! { #x.abs() })
            } else {
                None
            }
        }
        "random" => {
            if args.is_empty() {
                Some(quote! { rand::random::<f64>() })
            } else {
                None
            }
        }
        _ => None,
    }
}
