use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::func::{convert_expr, convert_expr_or_spread};

/// Handle console.* calls
pub fn handle(method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    match method {
        "log" => {
            let args_tokens: Vec<_> = args.iter().map(convert_expr_or_spread).collect();
            Some(quote! { println!(#(#args_tokens),*) })
        }
        "error" => {
            let args_tokens: Vec<_> = args.iter().map(convert_expr_or_spread).collect();
            Some(quote! { eprintln!(#(#args_tokens),*) })
        }
        _ => None,
    }
}
