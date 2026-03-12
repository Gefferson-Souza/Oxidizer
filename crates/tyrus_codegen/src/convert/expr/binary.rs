//! Binary expression code generation.
//!
//! Maps TypeScript binary operators to Rust equivalents.
//! Special handling: string concatenation (format!) vs numeric addition (+).

use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use crate::convert::helpers::is_string_expr;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    pub(crate) fn convert_bin_expr(&self, bin: &BinExpr) -> TokenStream {
        let left = self.convert_expr(&bin.left);
        let right = self.convert_expr(&bin.right);
        match bin.op {
            BinaryOp::EqEq | BinaryOp::EqEqEq => quote! { #left == #right },
            BinaryOp::NotEq | BinaryOp::NotEqEq => quote! { #left != #right },
            BinaryOp::Add => {
                if is_string_expr(&bin.left) || is_string_expr(&bin.right) {
                    quote! { format!("{}{}", #left, #right) }
                } else {
                    quote! { #left + #right }
                }
            }
            BinaryOp::Sub => quote! { #left - #right },
            BinaryOp::Mul => quote! { #left * #right },
            BinaryOp::Div => quote! { #left / #right },
            BinaryOp::Mod => quote! { #left % #right },
            BinaryOp::Lt => quote! { #left < #right },
            BinaryOp::LtEq => quote! { #left <= #right },
            BinaryOp::Gt => quote! { #left > #right },
            BinaryOp::GtEq => quote! { #left >= #right },
            BinaryOp::LogicalOr => quote! { #left || #right },
            BinaryOp::LogicalAnd => quote! { #left && #right },
            BinaryOp::NullishCoalescing => quote! { #left.unwrap_or(#right) },
            _ => {
                let op_str = format!("{:?}", bin.op);
                quote! { compile_error!(concat!("Tyrus: unsupported binary operator: ", #op_str)) }
            }
        }
    }
}
