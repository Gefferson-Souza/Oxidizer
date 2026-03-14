use std::collections::HashSet;

use quote::{format_ident, quote};
use swc_ecma_ast::{AssignTarget, Constructor, Expr, ExprStmt, Pat, Stmt};

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;
use crate::convert::type_mapper::map_ts_type;

/// Context for constructor conversion, replacing many positional arguments.
pub(crate) struct ConstructorCtx<'a> {
    pub(crate) constructor: &'a Constructor,
    pub(crate) class_fields: &'a [(String, bool)],
    pub(crate) has_generics: bool,
    pub(crate) generic_params: &'a HashSet<String>,
    pub(crate) dependency_fields: &'a HashSet<String>,
    pub(crate) is_service_or_controller: bool,
}

/// Check whether a type annotation refers to a dependency (non-primitive TypeRef).
pub(crate) fn is_dependency_type(
    type_ann: Option<&swc_ecma_ast::TsTypeAnn>,
    generic_params: &HashSet<String>,
) -> bool {
    let ann = match type_ann {
        Some(a) => a,
        None => return false,
    };
    let type_ref = match ann.type_ann.as_ts_type_ref() {
        Some(r) => r,
        None => return false,
    };
    match type_ref.type_name.as_ident() {
        Some(ident) => {
            let name = ident.sym.as_str();
            if generic_params.contains(name) {
                return false;
            }
            !matches!(
                name,
                "String" | "f64" | "bool" | "i32" | "Vec" | "Option" | "Array"
            )
        }
        None => true,
    }
}

/// Result of extracting constructor parameters from `TsParamProp` and `Param` nodes.
struct ExtractedParams {
    params: Vec<proc_macro2::TokenStream>,
    dependency_params: HashSet<String>,
    initialized_fields: HashSet<String>,
    field_inits: Vec<proc_macro2::TokenStream>,
}

/// Extract constructor parameters and auto-field-inits from `TsParamProp` / `Param` nodes.
fn extract_constructor_params(
    constructor: &Constructor,
    generic_params: &HashSet<String>,
) -> ExtractedParams {
    let mut result = ExtractedParams {
        params: Vec::new(),
        dependency_params: HashSet::new(),
        initialized_fields: HashSet::new(),
        field_inits: Vec::new(),
    };

    for param in &constructor.params {
        match param {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) => {
                extract_ts_param_prop(prop, generic_params, &mut result);
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(pat_param) => {
                extract_plain_param(pat_param, generic_params, &mut result);
            }
        }
    }

    result
}

/// Handle a `TsParamProp` — creates both a parameter and a field initializer.
fn extract_ts_param_prop(
    prop: &swc_ecma_ast::TsParamProp,
    generic_params: &HashSet<String>,
    result: &mut ExtractedParams,
) {
    let ident = match &prop.param {
        swc_ecma_ast::TsParamPropParam::Ident(id) => id,
        _ => return,
    };

    let param_name_str = ident.sym.to_string();
    let param_name = format_ident!("{}", to_snake_case(&param_name_str));
    let type_ann = ident.type_ann.as_ref();
    let mut param_type = map_ts_type(type_ann);

    if is_dependency_type(type_ann.map(std::convert::AsRef::as_ref), generic_params) {
        param_type = quote! { std::sync::Arc<#param_type> };
        result.dependency_params.insert(param_name_str.clone());
    }

    result.params.push(quote! { #param_name: #param_type });
    result.field_inits.push(quote! { #param_name: #param_name });
    result.initialized_fields.insert(param_name_str);
}

/// Handle a plain `Param` — creates a parameter but no auto-field-init.
fn extract_plain_param(
    pat_param: &swc_ecma_ast::Param,
    generic_params: &HashSet<String>,
    result: &mut ExtractedParams,
) {
    let ident = match &pat_param.pat {
        Pat::Ident(id) => id,
        _ => return,
    };

    let param_name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
    let mut param_type = map_ts_type(ident.type_ann.as_ref());

    if is_dependency_type(ident.type_ann.as_deref(), generic_params) {
        param_type = quote! { std::sync::Arc<#param_type> };
        result.dependency_params.insert(ident.sym.to_string());
    }

    result.params.push(quote! { #param_name: #param_type });
}

/// Context for extracting field initializations from the constructor body.
struct FieldInitCtx<'a> {
    class_fields: &'a [(String, bool)],
    dependency_fields: &'a HashSet<String>,
    dependency_params: &'a HashSet<String>,
    is_service_or_controller: bool,
}

/// Extract field assignments (`this.field = value`) and `super()` calls from the body.
fn extract_field_inits(
    generator: &RustGenerator,
    constructor: &Constructor,
    ctx: &FieldInitCtx<'_>,
    field_inits: &mut Vec<proc_macro2::TokenStream>,
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

/// Map `super(args)` arguments to parent fields by position.
fn extract_super_call(
    generator: &RustGenerator,
    call: &swc_ecma_ast::CallExpr,
    class_fields: &[(String, bool)],
    field_inits: &mut Vec<proc_macro2::TokenStream>,
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

/// Extract a `this.field = value` assignment into a field initializer.
fn extract_this_assign(
    generator: &RustGenerator,
    assign: &swc_ecma_ast::AssignExpr,
    ctx: &FieldInitCtx<'_>,
    field_inits: &mut Vec<proc_macro2::TokenStream>,
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

/// Wrap a field value in Arc / Arc<Mutex> as needed.
fn wrap_field_value(
    rhs: &Expr,
    field_name_str: &str,
    value: &proc_macro2::TokenStream,
    ctx: &FieldInitCtx<'_>,
) -> proc_macro2::TokenStream {
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

/// Fill missing optional fields with `None` and add `PhantomData` if generic.
fn fill_missing_fields(
    class_fields: &[(String, bool)],
    has_generics: bool,
    initialized_fields: &HashSet<String>,
    field_inits: &mut Vec<proc_macro2::TokenStream>,
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

/// Build the `new_di(...)` DI constructor from constructor params.
fn build_di_constructor(
    constructor: &Constructor,
    ctx: &ConstructorCtx<'_>,
) -> proc_macro2::TokenStream {
    let mut di_params = Vec::new();
    let mut di_field_inits = Vec::new();
    let mut di_initialized = HashSet::new();

    for param in &constructor.params {
        match param {
            swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) => {
                build_di_ts_param(
                    prop,
                    ctx.generic_params,
                    &mut di_params,
                    &mut di_field_inits,
                    &mut di_initialized,
                );
            }
            swc_ecma_ast::ParamOrTsParamProp::Param(pat_param) => {
                build_di_plain_param(
                    pat_param,
                    ctx,
                    &mut di_params,
                    &mut di_field_inits,
                    &mut di_initialized,
                );
            }
        }
    }

    if ctx.has_generics {
        di_field_inits.push(quote! { _marker: std::marker::PhantomData });
    }

    fill_di_defaults(
        ctx.class_fields,
        ctx.is_service_or_controller,
        &di_initialized,
        &mut di_field_inits,
    );

    quote! {
        pub fn new_di(#(#di_params),*) -> Self {
            Self {
                #(#di_field_inits),*
            }
        }
    }
}

/// Process a `TsParamProp` for the DI constructor.
fn build_di_ts_param(
    prop: &swc_ecma_ast::TsParamProp,
    generic_params: &HashSet<String>,
    di_params: &mut Vec<proc_macro2::TokenStream>,
    di_field_inits: &mut Vec<proc_macro2::TokenStream>,
    di_initialized: &mut HashSet<String>,
) {
    let ident = match &prop.param {
        swc_ecma_ast::TsParamPropParam::Ident(id) => id,
        _ => return,
    };

    let param_name_str = ident.sym.to_string();
    let param_name = format_ident!("{}", to_snake_case(&param_name_str));
    let type_ann = ident.type_ann.as_ref();
    let mut param_type = map_ts_type(type_ann);

    if is_dependency_type(type_ann.map(std::convert::AsRef::as_ref), generic_params) {
        param_type = quote! { std::sync::Arc<#param_type> };
    }

    di_params.push(quote! { #param_name: #param_type });
    di_field_inits.push(quote! { #param_name: #param_name });
    di_initialized.insert(param_name_str);
}

/// Process a plain `Param` for the DI constructor.
fn build_di_plain_param(
    pat_param: &swc_ecma_ast::Param,
    ctx: &ConstructorCtx<'_>,
    di_params: &mut Vec<proc_macro2::TokenStream>,
    di_field_inits: &mut Vec<proc_macro2::TokenStream>,
    di_initialized: &mut HashSet<String>,
) {
    let ident = match &pat_param.pat {
        Pat::Ident(id) => id,
        _ => return,
    };

    let param_name_str = ident.sym.to_string();
    let param_name = format_ident!("{}", &param_name_str);
    let param_type = map_ts_type(ident.type_ann.as_ref());

    if !is_dependency_type(ident.type_ann.as_deref(), ctx.generic_params) {
        return;
    }

    let arc_type = quote! { std::sync::Arc<#param_type> };
    di_params.push(quote! { #param_name: #arc_type });

    if ctx.class_fields.iter().any(|(n, _)| n == &param_name_str) {
        di_field_inits.push(quote! { #param_name: #param_name });
        di_initialized.insert(param_name_str);
    }
}

/// Fill uninitialized DI fields with `Default::default()`.
fn fill_di_defaults(
    class_fields: &[(String, bool)],
    is_service_or_controller: bool,
    di_initialized: &HashSet<String>,
    di_field_inits: &mut Vec<proc_macro2::TokenStream>,
) {
    for (name, _) in class_fields {
        if !di_initialized.contains(name) {
            let field_name = format_ident!("{}", to_snake_case(name));
            if is_service_or_controller {
                di_field_inits.push(
                    quote! { #field_name: std::sync::Arc::new(std::sync::Mutex::new(Default::default())) },
                );
            } else {
                di_field_inits.push(quote! { #field_name: Default::default() });
            }
        }
    }
}

impl RustGenerator {
    /// Dispatcher: convert a TypeScript constructor into `new()` + `new_di()` methods.
    pub(crate) fn convert_constructor(
        &self,
        _struct_name: &proc_macro2::Ident,
        ctx: &ConstructorCtx<'_>,
    ) -> proc_macro2::TokenStream {
        let mut extracted = extract_constructor_params(ctx.constructor, ctx.generic_params);

        let init_ctx = FieldInitCtx {
            class_fields: ctx.class_fields,
            dependency_fields: ctx.dependency_fields,
            dependency_params: &extracted.dependency_params,
            is_service_or_controller: ctx.is_service_or_controller,
        };

        extract_field_inits(
            self,
            ctx.constructor,
            &init_ctx,
            &mut extracted.field_inits,
            &mut extracted.initialized_fields,
        );

        fill_missing_fields(
            ctx.class_fields,
            ctx.has_generics,
            &extracted.initialized_fields,
            &mut extracted.field_inits,
        );

        let params = &extracted.params;
        let field_inits = &extracted.field_inits;

        if !field_inits.is_empty() {
            let di_tokens = build_di_constructor(ctx.constructor, ctx);

            quote! {
                pub fn new(#(#params),*) -> Self {
                    Self {
                        #(#field_inits),*
                    }
                }

                #di_tokens
            }
        } else {
            quote! {
                pub fn new(#(#params),*) -> Self {
                    compile_error!("Tyrus: complex constructor pattern not yet supported")
                }
            }
        }
    }
}
