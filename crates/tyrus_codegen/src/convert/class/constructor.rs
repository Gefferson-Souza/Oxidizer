use quote::{format_ident, quote};
use swc_ecma_ast::{AssignTarget, Constructor, Expr, ExprStmt, Pat, Stmt};

use crate::convert::helpers::to_snake_case;
use crate::convert::interface::RustGenerator;
use crate::convert::type_mapper::map_ts_type;

impl RustGenerator {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn convert_constructor(
        &self,
        _struct_name: &proc_macro2::Ident,
        constructor: &Constructor,
        class_fields: &[(String, bool)],
        has_generics: bool,
        generic_params: &std::collections::HashSet<String>,
        dependency_fields: &std::collections::HashSet<String>,
        is_service_or_controller: bool,
    ) -> proc_macro2::TokenStream {
        let mut params = Vec::new();
        let mut field_inits = Vec::new();
        let mut initialized_fields = std::collections::HashSet::new();
        let mut dependency_params: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for param in &constructor.params {
            match param {
                swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) => {
                    if let swc_ecma_ast::TsParamPropParam::Ident(ident) = &prop.param {
                        let param_name_str = ident.sym.to_string();
                        let param_name = format_ident!("{}", to_snake_case(&param_name_str));

                        let type_ann = ident.type_ann.as_ref();
                        let mut param_type = map_ts_type(type_ann);

                        // Heuristic: If it's a TypeRef (not primitive), wrap in Arc
                        let is_dependency = if let Some(ann) = type_ann {
                            if let Some(type_ref) = ann.type_ann.as_ts_type_ref() {
                                if let Some(ident) = type_ref.type_name.as_ident() {
                                    let name = ident.sym.as_str();
                                    if generic_params.contains(name) {
                                        false
                                    } else {
                                        !matches!(
                                            name,
                                            "String"
                                                | "f64"
                                                | "bool"
                                                | "i32"
                                                | "Vec"
                                                | "Option"
                                                | "Array"
                                        )
                                    }
                                } else {
                                    true
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_dependency {
                            param_type = quote! { std::sync::Arc<#param_type> };
                            dependency_params.insert(param_name_str.clone());
                        }

                        params.push(quote! { #param_name: #param_type });
                        field_inits.push(quote! { #param_name: #param_name });
                        initialized_fields.insert(param_name_str);
                    }
                }
                swc_ecma_ast::ParamOrTsParamProp::Param(pat_param) => {
                    if let Pat::Ident(ident) = &pat_param.pat {
                        let param_name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                        let mut param_type = map_ts_type(ident.type_ann.as_ref());

                        // Check dependency
                        let is_dependency = if let Some(ann) = ident.type_ann.as_ref() {
                            if let Some(type_ref) = ann.type_ann.as_ts_type_ref() {
                                if let Some(ident) = type_ref.type_name.as_ident() {
                                    let name = ident.sym.as_str();
                                    if generic_params.contains(name) {
                                        false
                                    } else {
                                        !matches!(
                                            name,
                                            "String"
                                                | "f64"
                                                | "bool"
                                                | "i32"
                                                | "Vec"
                                                | "Option"
                                                | "Array"
                                        )
                                    }
                                } else {
                                    true
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_dependency {
                            param_type = quote! { std::sync::Arc<#param_type> };
                            dependency_params.insert(ident.sym.to_string());
                        }

                        params.push(quote! { #param_name: #param_type });
                    }
                }
            }
        }

        // Try to extract field assignments from constructor body
        if let Some(body) = &constructor.body {
            for stmt in &body.stmts {
                if let Stmt::Expr(ExprStmt { expr, .. }) = stmt {
                    if let Expr::Assign(assign) = &**expr {
                        // Check if left side is this.field
                        if let AssignTarget::Simple(simple) = &assign.left {
                            if let Some(member) = simple.as_member() {
                                if member.obj.is_this() {
                                    if let Some(prop_ident) = member.prop.as_ident() {
                                        let field_name_str = prop_ident.sym.to_string();
                                        let field_name =
                                            format_ident!("{}", to_snake_case(&field_name_str));
                                        let value = self.convert_expr(&assign.right);

                                        let is_optional = class_fields
                                            .iter()
                                            .find(|(n, _)| n == &field_name_str)
                                            .map(|(_, opt)| *opt)
                                            .unwrap_or(false);

                                        let value = if is_optional {
                                            quote! { Some(#value) }
                                        } else {
                                            value
                                        };

                                        let value = if dependency_fields.contains(&field_name_str) {
                                            // Check if the assigned value is a parameter that is already a dependency
                                            let is_already_wrapped = if let Expr::Ident(ident) =
                                                &*assign.right
                                            {
                                                dependency_params.contains(&ident.sym.to_string())
                                            } else {
                                                false
                                            };

                                            if is_already_wrapped {
                                                value
                                            } else {
                                                quote! { std::sync::Arc::new(#value) }
                                            }
                                        } else if is_service_or_controller {
                                            // State field: Wrap in Arc::new(Mutex::new(value))
                                            quote! { std::sync::Arc::new(std::sync::Mutex::new(#value)) }
                                        } else {
                                            // Plain field initialization
                                            value
                                        };

                                        field_inits.push(quote! { #field_name: #value });
                                        initialized_fields.insert(field_name_str);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fill in missing optional fields with None
        for (name, is_optional) in class_fields {
            if *is_optional && !initialized_fields.contains(name) {
                let field_name = format_ident!("{}", to_snake_case(name));
                field_inits.push(quote! { #field_name: None });
            }
        }

        // Initialize _marker if generics exist
        if has_generics {
            field_inits.push(quote! { _marker: std::marker::PhantomData });
        }

        if !field_inits.is_empty() {
            // Generate new_di (Dependency Injection constructor)
            // It takes only dependencies and defaults primitives
            let mut di_params = Vec::new();
            let mut di_field_inits = Vec::new();
            let mut di_initialized_fields = std::collections::HashSet::new();

            for param in &constructor.params {
                match param {
                    swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) => {
                        if let swc_ecma_ast::TsParamPropParam::Ident(ident) = &prop.param {
                            let param_name_str = ident.sym.to_string();
                            let param_name = format_ident!("{}", to_snake_case(&param_name_str));
                            let type_ann = ident.type_ann.as_ref();
                            let mut param_type = map_ts_type(type_ann);

                            // Check if it's a dependency (not primitive/std type)
                            let is_dependency = if let Some(ann) = type_ann {
                                if let Some(type_ref) = ann.type_ann.as_ts_type_ref() {
                                    if let Some(ident) = type_ref.type_name.as_ident() {
                                        let name = ident.sym.as_str();
                                        if generic_params.contains(name) {
                                            false
                                        } else {
                                            !matches!(
                                                name,
                                                "String"
                                                    | "f64"
                                                    | "bool"
                                                    | "i32"
                                                    | "Vec"
                                                    | "Option"
                                                    | "Array"
                                            )
                                        }
                                    } else {
                                        true
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            };

                            if is_dependency {
                                param_type = quote! { std::sync::Arc<#param_type> };
                                di_params.push(quote! { #param_name: #param_type });
                                di_field_inits.push(quote! { #param_name: #param_name });
                            } else {
                                di_params.push(quote! { #param_name: #param_type });
                                di_field_inits.push(quote! { #param_name: #param_name });
                            }
                            // Always mark as initialized for TsParamProp as it creates a field
                            di_initialized_fields.insert(ident.sym.to_string());
                        }
                    }
                    swc_ecma_ast::ParamOrTsParamProp::Param(pat_param) => {
                        if let Pat::Ident(ident) = &pat_param.pat {
                            let param_name_str = ident.sym.to_string();
                            let param_name = format_ident!("{}", param_name_str);
                            let param_type = map_ts_type(ident.type_ann.as_ref());

                            // Check dependency using same logic
                            let is_dependency = if let Some(ann) = ident.type_ann.as_ref() {
                                if let Some(type_ref) = ann.type_ann.as_ts_type_ref() {
                                    if let Some(ident) = type_ref.type_name.as_ident() {
                                        let name = ident.sym.as_str();
                                        if generic_params.contains(name) {
                                            false
                                        } else {
                                            !matches!(
                                                name,
                                                "String"
                                                    | "f64"
                                                    | "bool"
                                                    | "i32"
                                                    | "Vec"
                                                    | "Option"
                                                    | "Array"
                                            )
                                        }
                                    } else {
                                        true
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            };

                            if is_dependency {
                                let arc_type = quote! { std::sync::Arc<#param_type> };
                                di_params.push(quote! { #param_name: #arc_type });

                                // If this param matches a class field, initialize it
                                if class_fields.iter().any(|(n, _)| n == &param_name_str) {
                                    di_field_inits.push(quote! { #param_name: #param_name });
                                    di_initialized_fields.insert(param_name_str);
                                }
                            }
                            // If not dependency, we don't add to params, and we don't add to inits.
                            // We also do NOT add to di_initialized_fields, so it gets Default::default() later.
                        }
                    }
                }
            }

            if has_generics {
                di_field_inits.push(quote! { _marker: std::marker::PhantomData });
            }

            for (name, _) in class_fields {
                if !di_initialized_fields.contains(name) {
                    let field_name = format_ident!("{}", to_snake_case(name));
                    if is_service_or_controller {
                        di_field_inits.push(quote! { #field_name: std::sync::Arc::new(std::sync::Mutex::new(Default::default())) });
                    } else {
                        di_field_inits.push(quote! { #field_name: Default::default() });
                    }
                }
            }

            quote! {
                pub fn new(#(#params),*) -> Self {
                    Self {
                        #(#field_inits),*
                    }
                }

                pub fn new_di(#(#di_params),*) -> Self {
                    Self {
                        #(#di_field_inits),*
                    }
                }
            }
        } else {
            // Fallback if we can't parse constructor body
            quote! {
                pub fn new(#(#params),*) -> Self {
                    compile_error!("Tyrus: complex constructor pattern not yet supported")
                }
            }
        }
    }
}
