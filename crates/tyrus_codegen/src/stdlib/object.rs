use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

/// Handle Object static method calls (Object.keys, Object.values, Object.entries)
pub(crate) fn handle(gen: &RustGenerator, method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    match method {
        "keys" => {
            if args.len() == 1 {
                let obj = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #obj.keys().cloned().collect::<Vec<_>>()
                })
            } else {
                None
            }
        }
        "values" => {
            if args.len() == 1 {
                let obj = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #obj.values().cloned().collect::<Vec<_>>()
                })
            } else {
                None
            }
        }
        "entries" => {
            if args.len() == 1 {
                let obj = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()
                })
            } else {
                None
            }
        }
        _ => None,
    }
}
