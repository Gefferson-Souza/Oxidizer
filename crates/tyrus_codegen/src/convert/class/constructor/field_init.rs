//! Constructor body → field initializer extraction.
//!
//! Walks the constructor body's statements:
//!
//! - `super(args)` calls map positional args to inherited parent fields
//! - `this.field = value` assignments become field initializers
//! - Optional fields are wrapped in `Some(_)`, dependency fields in
//!   `Arc::new(_)`, and service/controller state fields in
//!   `Arc::new(Mutex::new(_))`.
//!
//! Finally, any remaining optional fields not initialized in the body are
//! filled with `None`, and `_marker: PhantomData` is appended for generics.

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{AssignTarget, Constructor, Expr, ExprStmt, Stmt};

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;

pub(super) struct FieldInitCtx<'a> {
    pub class_fields: &'a [(String, bool)],
    pub dependency_fields: &'a HashSet<String>,
    pub dependency_params: &'a HashSet<String>,
    pub is_service_or_controller: bool,
}

pub(super) fn extract_field_inits(
    generator: &RustGenerator,
    constructor: &Constructor,
    ctx: &FieldInitCtx<'_>,
    field_inits: &mut Vec<TokenStream>,
    initialized_fields: &mut HashSet<String>,
) {
    let body = match &constructor.body {
        Some(b) => b,
        None => return,
    };

    for stmt in &body.stmts {
        let expr_stmt = match stmt {
            Stmt::Expr(ExprStmt { expr, .. }) => expr,
            _ => continue,
        };

        if let Expr::Call(call) = &**expr_stmt {
            if let swc_ecma_ast::Callee::Super(_) = &call.callee {
                extract_super_call(
                    generator,
                    call,
                    ctx.class_fields,
                    field_inits,
                    initialized_fields,
                );
                continue;
            }
        }

        if let Expr::Assign(assign) = &**expr_stmt {
            extract_this_assign(generator, assign, ctx, field_inits, initialized_fields);
        }
    }
}

fn extract_super_call(
    generator: &RustGenerator,
    call: &swc_ecma_ast::CallExpr,
    class_fields: &[(String, bool)],
    field_inits: &mut Vec<TokenStream>,
    initialized_fields: &mut HashSet<String>,
) {
    let parent_fields: Vec<&String> = class_fields
        .iter()
        .filter(|(name, _)| !initialized_fields.contains(name.as_str()))
        .map(|(n, _)| n)
        .collect();

    for (idx, arg) in call.args.iter().enumerate() {
        if idx < parent_fields.len() {
            let field_name_str = parent_fields[idx];
            let field_name = format_ident!("{}", to_snake_case(field_name_str));
            let value = generator.convert_expr(&arg.expr);
            field_inits.push(quote! { #field_name: #value });
            initialized_fields.insert(field_name_str.clone());
        }
    }
}

fn extract_this_assign(
    generator: &RustGenerator,
    assign: &swc_ecma_ast::AssignExpr,
    ctx: &FieldInitCtx<'_>,
    field_inits: &mut Vec<TokenStream>,
    initialized_fields: &mut HashSet<String>,
) {
    let simple = match &assign.left {
        AssignTarget::Simple(s) => s,
        _ => return,
    };
    let member = match simple.as_member() {
        Some(m) if m.obj.is_this() => m,
        _ => return,
    };
    let prop_ident = match member.prop.as_ident() {
        Some(id) => id,
        None => return,
    };

    let field_name_str = prop_ident.sym.to_string();
    let field_name = format_ident!("{}", to_snake_case(&field_name_str));
    let value = generator.convert_expr(&assign.right);

    let is_optional = ctx
        .class_fields
        .iter()
        .find(|(n, _)| n == &field_name_str)
        .map(|(_, opt)| *opt)
        .unwrap_or(false);

    let value = if is_optional {
        quote! { Some(#value) }
    } else {
        value
    };

    let value = wrap_field_value(&assign.right, &field_name_str, &value, ctx);

    field_inits.push(quote! { #field_name: #value });
    initialized_fields.insert(field_name_str);
}

fn wrap_field_value(
    rhs: &Expr,
    field_name_str: &str,
    value: &TokenStream,
    ctx: &FieldInitCtx<'_>,
) -> TokenStream {
    if ctx.dependency_fields.contains(field_name_str) {
        let is_already_wrapped = if let Expr::Ident(ident) = rhs {
            ctx.dependency_params.contains(&ident.sym.to_string())
        } else {
            false
        };
        if is_already_wrapped {
            value.clone()
        } else {
            quote! { std::sync::Arc::new(#value) }
        }
    } else if ctx.is_service_or_controller {
        quote! { std::sync::Arc::new(std::sync::Mutex::new(#value)) }
    } else {
        value.clone()
    }
}

pub(super) fn fill_missing_fields(
    class_fields: &[(String, bool)],
    has_generics: bool,
    initialized_fields: &HashSet<String>,
    field_inits: &mut Vec<TokenStream>,
) {
    for (name, is_optional) in class_fields {
        if *is_optional && !initialized_fields.contains(name) {
            let field_name = format_ident!("{}", to_snake_case(name));
            field_inits.push(quote! { #field_name: None });
        }
    }
    if has_generics {
        field_inits.push(quote! { _marker: std::marker::PhantomData });
    }
}
