mod constructor;
mod method;
mod mutation;
mod routing;

use quote::{format_ident, quote};
use swc_ecma_ast::{ClassDecl, ClassMember, Constructor};

use super::helpers::to_snake_case;
use super::interface::RustGenerator;
use super::type_mapper::{is_optional_type, map_ts_type};

impl RustGenerator {
    pub fn process_class_decl(&mut self, n: &ClassDecl) {
        let class_name = n.ident.sym.to_string();
        let struct_name = format_ident!("{}", class_name);

        self.is_controller = class_name.ends_with("Controller");

        let is_service_or_controller = class_name.ends_with("Service")
            || class_name.ends_with("Controller")
            || self.is_controller;

        // Extract generic params early
        let mut generic_params = std::collections::HashSet::new();
        if let Some(type_params) = &n.class.type_params {
            for param in &type_params.params {
                generic_params.insert(param.name.sym.to_string());
            }
        }

        // Detect inheritance: class Dog extends Animal
        let parent_class_name = n.class.super_class.as_ref().and_then(|expr| {
            if let swc_ecma_ast::Expr::Ident(ident) = &**expr {
                Some(ident.sym.to_string())
            } else {
                None
            }
        });

        // 1. Generate Struct (Properties)
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut constructor: Option<&Constructor> = None;
        let mut class_fields_meta = Vec::new();
        let mut own_field_names: Vec<(String, proc_macro2::TokenStream, bool)> = Vec::new();

        // If extending a parent, include parent fields first
        if let Some(ref parent) = parent_class_name {
            if let Some(parent_fields) = self.class_fields.get(parent) {
                for (field_name, field_type, is_opt) in parent_fields {
                    let fname = format_ident!("{}", to_snake_case(field_name));
                    let ftype = field_type.clone();
                    fields.push(quote! { pub #fname: #ftype });
                    class_fields_meta.push((field_name.clone(), *is_opt));
                    own_field_names.push((field_name.clone(), ftype, *is_opt));
                }
            }
        }

        let mut dependency_fields = std::collections::HashSet::new();
        self.current_class_state_fields.clear();

        // Pass 1: Process Properties (to populate state fields map)
        for member in &n.class.body {
            if let ClassMember::ClassProp(prop) = member {
                if let Some((field_tokens, name, is_opt, is_dep, type_str)) =
                    self.convert_prop(prop, &generic_params, is_service_or_controller)
                {
                    fields.push(field_tokens);
                    class_fields_meta.push((name.clone(), is_opt));
                    // Store raw type as TokenStream for inheritance
                    let raw_type_tokens = map_ts_type(prop.type_ann.as_ref());
                    own_field_names.push((name.clone(), raw_type_tokens, is_opt));
                    if is_dep {
                        dependency_fields.insert(name);
                    } else if is_service_or_controller {
                        // Store the type string to check for primitives later
                        self.current_class_state_fields.insert(name, type_str);
                    }
                }
            }
        }

        // Pass 2: Process Methods and Constructor
        let mut static_method_names = std::collections::HashSet::new();
        for member in &n.class.body {
            match member {
                ClassMember::Method(method) => {
                    if method.is_static {
                        if let Some(ident) = method.key.as_ident() {
                            static_method_names.insert(ident.sym.to_string());
                        }
                    }
                    methods.push(method);
                }
                ClassMember::Constructor(cons) => {
                    constructor = Some(cons);
                }
                _ => {}
            }
        }

        // Collect fields from constructor (private/public params)
        if let Some(cons) = constructor {
            for param in &cons.params {
                if let swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) = param {
                    if let swc_ecma_ast::TsParamPropParam::Ident(ident) = &prop.param {
                        let field_name_str = ident.sym.to_string();
                        let field_name = format_ident!("{}", to_snake_case(&field_name_str));

                        let type_ann = ident.type_ann.as_ref();
                        let mut field_type = map_ts_type(type_ann);
                        let raw_type_str = field_type.to_string();

                        // Heuristic: If it's a TypeRef (not primitive), wrap in Arc
                        let is_dependency = if let Some(ann) = type_ann {
                            if let Some(type_ref) = ann.type_ann.as_ts_type_ref() {
                                if let Some(ident) = type_ref.type_name.as_ident() {
                                    let name = ident.sym.as_str();
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
                            field_type = quote! { std::sync::Arc<#field_type> };
                            dependency_fields.insert(field_name_str.clone());
                        } else if is_service_or_controller {
                            // State field: Wrap in Arc<Mutex<T>> for Interior Mutability
                            field_type = quote! { std::sync::Arc<std::sync::Mutex<#field_type>> };
                            self.current_class_state_fields
                                .insert(field_name_str.clone(), raw_type_str);
                        } else {
                            // DTO field: Raw type, no mutex
                        }

                        fields.push(quote! { pub #field_name: #field_type });
                    }
                }
            }
        }

        // All structs are pub — needed for main.rs to reference controllers/services
        let vis = quote! { pub };

        let (generics_struct_decl, generics_impl_decl, generics_use) = if let Some(type_params) =
            &n.class.type_params
        {
            let params_struct: Vec<_> = type_params
                .params
                .iter()
                .map(|p| {
                    let name = p.name.sym.to_string();
                    format_ident!("{}", name)
                })
                .collect();

            let params_impl: Vec<_> = type_params
                .params
                .iter()
                .map(|p| {
                    let name = p.name.sym.to_string();
                    let ident = format_ident!("{}", name);
                    quote! { #ident: serde::de::DeserializeOwned + serde::Serialize + Clone + Default + std::fmt::Debug }
                })
                .collect();

            let params_use: Vec<_> = type_params
                .params
                .iter()
                .map(|p| format_ident!("{}", p.name.sym.to_string()))
                .collect();

            // Add PhantomData to usage to avoid unused type parameter error
            if !params_use.is_empty() {
                let phantom_type = if params_use.len() == 1 {
                    quote! { #(#params_use)* }
                } else {
                    quote! { (#(#params_use),*) }
                };
                fields.push(quote! {
                    #[serde(skip)]
                    pub _marker: std::marker::PhantomData<#phantom_type>
                });
            }

            (
                quote! { <#(#params_struct),*> },
                quote! { <#(#params_impl),*> },
                quote! { <#(#params_use),*> },
            )
        } else {
            (quote! {}, quote! {}, quote! {})
        };

        let mut derives = vec![quote! { Default }, quote! { Debug }, quote! { Clone }];
        if !is_service_or_controller {
            derives.push(quote! { PartialEq });
            derives.push(quote! { serde::Serialize });
            derives.push(quote! { serde::Deserialize });
        }

        let derive_attr = quote! {
            #[derive(#( #derives ),*)]
        };

        let serde_attr = if !is_service_or_controller {
            quote! { #[serde(rename_all = "camelCase")] }
        } else {
            quote! {}
        };

        let struct_def = quote! {
            #derive_attr
            #serde_attr
            #vis struct #struct_name #generics_struct_decl {
                #(#fields),*
            }
        };

        self.code.push_str(&struct_def.to_string());
        self.code.push('\n');

        // Store field map for potential child classes
        self.class_fields
            .insert(class_name.clone(), own_field_names);

        // Store static method names for call-site transformation
        if !static_method_names.is_empty() {
            self.static_methods
                .insert(class_name.clone(), static_method_names);
        }

        // 2. Generate Impl (Methods)
        let mut impl_items = Vec::new();

        // Constructor
        if let Some(cons) = constructor {
            let ctx = constructor::ConstructorCtx {
                constructor: cons,
                class_fields: &class_fields_meta,
                has_generics: n.class.type_params.is_some(),
                generic_params: &generic_params,
                dependency_fields: &dependency_fields,
                is_service_or_controller,
            };
            let constructor_tokens = self.convert_constructor(&struct_name, &ctx);
            impl_items.push(constructor_tokens);
        } else {
            // Default constructor if none exists
            impl_items.push(quote! {
                pub fn new() -> Self {
                    Self::default()
                }
                pub fn new_di() -> Self {
                    Self::default()
                }
            });
        }

        // Methods
        let mut routes: Vec<(String, String, String)> = Vec::new();
        for method in methods {
            let (method_tokens, route_info) = self.convert_method(method, is_service_or_controller);
            impl_items.push(method_tokens);
            if let Some(info) = route_info {
                routes.push(info);
            }
        }

        // Generate controller routing if applicable
        let controller_info = routing::extract_controller_info(n);
        if controller_info.is_controller {
            self.emit_controller_routing(
                &struct_name,
                &class_name,
                &controller_info,
                &routes,
                &mut impl_items,
            );
        }

        let impl_block = quote! {
            impl #generics_impl_decl #struct_name #generics_use {
                #(#impl_items)*
            }
        };

        self.code.push_str(&impl_block.to_string());
        self.code.push('\n');
    }

    fn convert_prop(
        &self,
        prop: &swc_ecma_ast::ClassProp,
        generic_params: &std::collections::HashSet<String>,
        is_service_or_controller: bool,
    ) -> Option<(proc_macro2::TokenStream, String, bool, bool, String)> {
        let field_name_str = if let Some(ident) = prop.key.as_ident() {
            ident.sym.to_string()
        } else {
            return None;
        };
        let field_name = format_ident!("{}", to_snake_case(&field_name_str));
        let mut field_type = map_ts_type(prop.type_ann.as_ref());

        // Capture the raw inner type string before wrapping in Option/Arc
        let raw_type_str = field_type.to_string();

        // Check dependency
        let is_dependency = if let Some(ann) = prop.type_ann.as_ref() {
            if let Some(type_ref) = ann.type_ann.as_ts_type_ref() {
                if let Some(ident) = type_ref.type_name.as_ident() {
                    let name = ident.sym.as_str();
                    if generic_params.contains(name) {
                        false
                    } else {
                        !matches!(
                            name,
                            "String" | "f64" | "bool" | "i32" | "Vec" | "Option" | "Array"
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
            field_type = quote! { std::sync::Arc<#field_type> };
        } else if is_service_or_controller {
            // State field: Wrap in Arc<Mutex<T>> ONLY for services/controllers
            field_type = quote! { std::sync::Arc<std::sync::Mutex<#field_type>> };
        } else {
            // DTO/Entity field: Keep raw type (or map type normally)
        }

        let is_optional_union = is_optional_type(prop.type_ann.as_deref());
        let is_effectively_optional = prop.is_optional || is_optional_union;

        if prop.is_optional {
            // If it's optional via `?`, we wrap in Option.
            field_type = quote! { Option<#field_type> };
        }

        Some((
            quote! {
                pub #field_name: #field_type
            },
            field_name_str,
            is_effectively_optional,
            is_dependency,
            raw_type_str,
        ))
    }
}
