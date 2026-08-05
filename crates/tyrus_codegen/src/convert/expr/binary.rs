//! Binary expression code generation.
//!
//! Maps TypeScript binary operators to Rust equivalents.
//! Special handling: string concatenation (format!) vs numeric addition (+).
//! Optimization: chained string `+` is flattened into a single format!().

use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::{BinExpr, BinaryOp, Expr};

use crate::convert::helpers::is_string_expr;
use crate::convert::interface::RustGenerator;

impl RustGenerator {
    // Bound: SWC AST is finite; recursion depth ≤ AST depth (Rule 1, POWER_OF_TEN.md).
    pub(crate) fn convert_bin_expr(&self, bin: &BinExpr) -> TokenStream {
        if bin.op == BinaryOp::Add {
            if is_string_expr(&bin.left) || is_string_expr(&bin.right) {
                // Flatten chained string concatenation into a single format!()
                let mut parts = Vec::new();
                collect_string_concat_parts(self, &Expr::Bin(bin.clone()), &mut parts);
                let fmt_str = "{}".repeat(parts.len());
                quote! { format!(#fmt_str, #(#parts),*) }
            } else {
                let left = self.convert_expr(&bin.left);
                let right = self.convert_expr(&bin.right);
                quote! { #left + #right }
            }
        } else {
            let left = self.convert_expr(&bin.left);
            let right = self.convert_expr(&bin.right);
            match bin.op {
                BinaryOp::EqEq | BinaryOp::EqEqEq => quote! { #left == #right },
                BinaryOp::NotEq | BinaryOp::NotEqEq => quote! { #left != #right },
                BinaryOp::Sub => quote! { #left - #right },
                BinaryOp::Mul => quote! { #left * #right },
                BinaryOp::Div => quote! { #left / #right },
                BinaryOp::Mod => quote! { #left % #right },
                BinaryOp::Exp => quote! { (#left as f64).powf(#right as f64) },
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
}

/// Recursively collect all parts of a chained string concatenation.
/// `a + b + c + d` → [convert(a), convert(b), convert(c), convert(d)]
/// instead of nested format!("{}{}", format!("{}{}", ...), d)
/// Once we know we're inside a string concat chain, ALL child Add nodes are string parts.
// Bound: SWC AST is finite; recursion depth ≤ AST depth (Rule 1, POWER_OF_TEN.md).
fn collect_string_concat_parts(gen: &RustGenerator, expr: &Expr, parts: &mut Vec<TokenStream>) {
    if let Expr::Bin(bin) = expr {
        if bin.op == BinaryOp::Add {
            // Inside a known string-concat chain: recurse into both sides
            collect_string_concat_parts(gen, &bin.left, parts);
            collect_string_concat_parts(gen, &bin.right, parts);
            return;
        }
    }
    parts.push(gen.convert_expr(expr));
}
