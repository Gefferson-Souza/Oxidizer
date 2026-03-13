use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

/// Handle Math.* calls
pub fn handle(gen: &RustGenerator, method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    match method {
        "max" => {
            if args.len() == 1 && args[0].spread.is_some() {
                // Math.max(...arr)
                let arg = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #arg.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
                })
            } else if args.len() == 2 {
                if args[0].spread.is_some() {
                    // Math.max(...arr, val)
                    let arr = gen.convert_expr_or_spread(&args[0]);
                    let val = gen.convert_expr_or_spread(&args[1]);
                    Some(quote! {
                        #arr.iter().fold(#val, |a, &b| a.max(b))
                    })
                } else {
                    // Math.max(a, b)
                    let a = gen.convert_expr_or_spread(&args[0]);
                    let b = gen.convert_expr_or_spread(&args[1]);
                    Some(quote! { #a.max(#b) })
                }
            } else {
                None
            }
        }
        "min" => {
            if args.len() == 1 && args[0].spread.is_some() {
                // Math.min(...arr) -> arr.iter().fold(f64::INFINITY, |a, &b| a.min(b))
                let arg = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #arg.iter().fold(f64::INFINITY, |a, &b| a.min(b))
                })
            } else if args.len() == 2 {
                let a = gen.convert_expr_or_spread(&args[0]);
                let b = gen.convert_expr_or_spread(&args[1]);
                Some(quote! { #a.min(#b) })
            } else {
                None
            }
        }
        "round" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x).round() })
            } else {
                None
            }
        }
        "floor" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x).floor() })
            } else {
                None
            }
        }
        "ceil" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x).ceil() })
            } else {
                None
            }
        }
        "abs" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x).abs() })
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
        "pow" => {
            if args.len() == 2 {
                let base = gen.convert_expr_or_spread(&args[0]);
                let exp = gen.convert_expr_or_spread(&args[1]);
                Some(quote! { (#base as f64).powf(#exp as f64) })
            } else {
                None
            }
        }
        "sqrt" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x as f64).sqrt() })
            } else {
                None
            }
        }
        "log" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x as f64).ln() })
            } else {
                None
            }
        }
        "trunc" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x).trunc() })
            } else {
                None
            }
        }
        "sign" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                // JS Math.sign(0) returns 0, but Rust f64::signum() returns 1.0
                Some(quote! {
                    {
                        let __v = #x;
                        if __v == 0.0 { 0.0 } else { __v.signum() }
                    }
                })
            } else {
                None
            }
        }
        _ => None,
    }
}
