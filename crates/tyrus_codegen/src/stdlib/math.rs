use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

pub(crate) fn handle(
    gen: &RustGenerator,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    match method {
        "max" => handle_max(gen, args),
        "min" => handle_min(gen, args),
        "pow" => handle_pow(gen, args),
        "random" => handle_random(args),
        "sign" => handle_sign(gen, args),
        "round" => unary_method(gen, args, |x| quote! { (#x).round() }),
        "floor" => unary_method(gen, args, |x| quote! { (#x).floor() }),
        "ceil" => unary_method(gen, args, |x| quote! { (#x).ceil() }),
        "abs" => unary_method(gen, args, |x| quote! { (#x).abs() }),
        "sqrt" => unary_method(gen, args, |x| quote! { (#x as f64).sqrt() }),
        "log" => unary_method(gen, args, |x| quote! { (#x as f64).ln() }),
        "trunc" => unary_method(gen, args, |x| quote! { (#x).trunc() }),
        "sin" => unary_method(gen, args, |x| quote! { (#x as f64).sin() }),
        "cos" => unary_method(gen, args, |x| quote! { (#x as f64).cos() }),
        "tan" => unary_method(gen, args, |x| quote! { (#x as f64).tan() }),
        _ => None,
    }
}

fn unary_method(
    gen: &RustGenerator,
    args: &[ExprOrSpread],
    emit: impl FnOnce(TokenStream) -> TokenStream,
) -> Option<TokenStream> {
    if args.len() == 1 {
        Some(emit(gen.convert_expr_or_spread(&args[0])))
    } else {
        None
    }
}

fn handle_max(gen: &RustGenerator, args: &[ExprOrSpread]) -> Option<TokenStream> {
    if args.len() == 1 && args[0].spread.is_some() {
        let arg = gen.convert_expr_or_spread(&args[0]);
        Some(quote! { #arg.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)) })
    } else if args.len() == 2 {
        if args[0].spread.is_some() {
            let arr = gen.convert_expr_or_spread(&args[0]);
            let val = gen.convert_expr_or_spread(&args[1]);
            Some(quote! { #arr.iter().fold(#val, |a, &b| a.max(b)) })
        } else {
            let a = gen.convert_expr_or_spread(&args[0]);
            let b = gen.convert_expr_or_spread(&args[1]);
            Some(quote! { #a.max(#b) })
        }
    } else {
        None
    }
}

fn handle_min(gen: &RustGenerator, args: &[ExprOrSpread]) -> Option<TokenStream> {
    if args.len() == 1 && args[0].spread.is_some() {
        let arg = gen.convert_expr_or_spread(&args[0]);
        Some(quote! { #arg.iter().fold(f64::INFINITY, |a, &b| a.min(b)) })
    } else if args.len() == 2 {
        let a = gen.convert_expr_or_spread(&args[0]);
        let b = gen.convert_expr_or_spread(&args[1]);
        Some(quote! { #a.min(#b) })
    } else {
        None
    }
}

fn handle_pow(gen: &RustGenerator, args: &[ExprOrSpread]) -> Option<TokenStream> {
    if args.len() == 2 {
        let base = gen.convert_expr_or_spread(&args[0]);
        let exp = gen.convert_expr_or_spread(&args[1]);
        Some(quote! { (#base as f64).powf(#exp as f64) })
    } else {
        None
    }
}

fn handle_random(args: &[ExprOrSpread]) -> Option<TokenStream> {
    if args.is_empty() {
        Some(quote! { rand::random::<f64>() })
    } else {
        None
    }
}

fn handle_sign(gen: &RustGenerator, args: &[ExprOrSpread]) -> Option<TokenStream> {
    if args.len() == 1 {
        let x = gen.convert_expr_or_spread(&args[0]);
        Some(quote! {
            { let __v = #x; if __v == 0.0 { 0.0 } else { __v.signum() } }
        })
    } else {
        None
    }
}
