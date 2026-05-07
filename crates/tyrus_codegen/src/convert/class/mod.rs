mod constructor;
mod frame;
mod getter_setter;
mod method;
mod mutation;
mod routing;
mod state_field;

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{ClassDecl, ClassMember, ClassMethod, Constructor};

use super::helpers::to_snake_case;
use super::interface::RustGenerator;
use super::type_mapper::map_ts_type;
use frame::{
    compute_class_generics, extract_class_meta, struct_derives, ClassFields, ClassFrame,
    ClassMembers, ClassMeta,
};
use state_field::wrap_state_field_type;

impl RustGenerator {
    pub fn process_class_decl(&mut self, n: &ClassDecl) {
        let meta = extract_class_meta(n);
        let struct_name = format_ident!("{}", &meta.class_name);

        self.emit_guard_middleware_if_any(n, &meta.class_name);
        self.is_controller = meta.is_controller;
        self.current_class_state_fields.clear();

        let members = self.categorize_members(n);
        let mut fields = self.collect_class_fields(n, &meta, &members);
        let generics = compute_class_generics(n, &mut fields.fields);
        let frame = ClassFrame {
            decl: n,
            struct_name,
            meta,
            members,
            fields,
            generics,
        };

        self.emit_class_struct(&frame);
        self.store_class_metadata(&frame);
        self.emit_class_impl(&frame);
    }

    fn emit_guard_middleware_if_any(&mut self, n: &ClassDecl, class_name: &str) {
        if let Some(middleware) = self.try_emit_guard_middleware(n, class_name) {
            self.code.push_str(&middleware.to_string());
            self.code.push('\n');
        }
    }

    fn categorize_members<'a>(&mut self, n: &'a ClassDecl) -> ClassMembers<'a> {
        let mut acc = ClassMembers {
            methods: Vec::new(),
            getters: Vec::new(),
            setters: Vec::new(),
            constructor: None,
            static_method_names: HashSet::new(),
        };

        for member in &n.class.body {
            match member {
                ClassMember::Method(method) => self.classify_method(method, &mut acc),
                ClassMember::Constructor(cons) => acc.constructor = Some(cons),
                _ => {}
            }
        }
        acc
    }

    fn classify_method<'a>(&mut self, method: &'a ClassMethod, acc: &mut ClassMembers<'a>) {
        match method.kind {
            swc_ecma_ast::MethodKind::Getter => {
                if let Some(ident) = method.key.as_ident() {
                    self.getter_names.insert(ident.sym.to_string());
                }
                acc.getters.push(method);
            }
            swc_ecma_ast::MethodKind::Setter => {
                if let Some(ident) = method.key.as_ident() {
                    self.setter_names.insert(ident.sym.to_string());
                }
                acc.setters.push(method);
            }
            swc_ecma_ast::MethodKind::Method => {
                if method.is_static {
                    if let Some(ident) = method.key.as_ident() {
                        acc.static_method_names.insert(ident.sym.to_string());
                    }
                }
                acc.methods.push(method);
            }
        }
    }

    fn collect_class_fields(
        &mut self,
        n: &ClassDecl,
        meta: &ClassMeta,
        members: &ClassMembers<'_>,
    ) -> ClassFields {
        let mut acc = ClassFields {
            fields: Vec::new(),
            class_fields_meta: Vec::new(),
            own_field_names: Vec::new(),
            dependency_fields: HashSet::new(),
        };

        self.inherit_parent_fields(meta, &mut acc);
        self.collect_property_fields(n, meta, &mut acc);
        if let Some(cons) = members.constructor {
            self.collect_constructor_fields(cons, meta, &mut acc);
        }
        acc
    }

    fn inherit_parent_fields(&self, meta: &ClassMeta, acc: &mut ClassFields) {
        let Some(parent) = meta.parent_class_name.as_deref() else {
            return;
        };
        let Some(parent_fields) = self.class_fields.get(parent) else {
            return;
        };
        for (field_name, field_type, is_opt) in parent_fields {
            let fname = format_ident!("{}", to_snake_case(field_name));
            let ftype = field_type.clone();
            acc.fields.push(quote! { pub #fname: #ftype });
            acc.class_fields_meta.push((field_name.clone(), *is_opt));
            acc.own_field_names
                .push((field_name.clone(), ftype, *is_opt));
        }
    }

    fn collect_property_fields(&mut self, n: &ClassDecl, meta: &ClassMeta, acc: &mut ClassFields) {
        for member in &n.class.body {
            let ClassMember::ClassProp(prop) = member else {
                continue;
            };
            let Some((field_tokens, name, is_opt, is_dep, type_str)) = state_field::convert_prop(
                self,
                prop,
                &meta.generic_params,
                meta.is_service_or_controller,
            ) else {
                continue;
            };
            acc.fields.push(field_tokens);
            acc.class_fields_meta.push((name.clone(), is_opt));
            let raw_type_tokens = map_ts_type(prop.type_ann.as_ref());
            acc.own_field_names
                .push((name.clone(), raw_type_tokens, is_opt));
            if is_dep {
                acc.dependency_fields.insert(name);
            } else if meta.is_service_or_controller {
                self.current_class_state_fields.insert(name, type_str);
            }
        }
    }

    fn collect_constructor_fields(
        &mut self,
        cons: &Constructor,
        meta: &ClassMeta,
        acc: &mut ClassFields,
    ) {
        for param in &cons.params {
            let swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) = param else {
                continue;
            };
            let swc_ecma_ast::TsParamPropParam::Ident(ident) = &prop.param else {
                continue;
            };
            self.append_constructor_param_field(ident, meta, acc);
        }
    }

    fn append_constructor_param_field(
        &mut self,
        ident: &swc_ecma_ast::BindingIdent,
        meta: &ClassMeta,
        acc: &mut ClassFields,
    ) {
        let field_name_str = ident.sym.to_string();
        let field_name = format_ident!("{}", to_snake_case(&field_name_str));
        let type_ann = ident.type_ann.as_ref();
        let inner_type = map_ts_type(type_ann);
        let raw_type_str = inner_type.to_string();

        let is_dependency = meta.is_service_or_controller
            && constructor::is_dependency_type(
                type_ann.map(std::convert::AsRef::as_ref),
                &meta.generic_params,
            );

        if is_dependency {
            acc.dependency_fields.insert(field_name_str.clone());
        } else if meta.is_service_or_controller {
            self.current_class_state_fields
                .insert(field_name_str, raw_type_str);
        }

        let field_type =
            wrap_state_field_type(&inner_type, is_dependency, meta.is_service_or_controller);
        acc.fields.push(quote! { pub #field_name: #field_type });
    }

    fn emit_class_struct(&mut self, frame: &ClassFrame<'_>) {
        let derives = struct_derives(frame.meta.is_service_or_controller);
        let derive_attr = quote! { #[derive(#( #derives ),*)] };
        let serde_attr = if frame.meta.is_service_or_controller {
            quote! {}
        } else {
            quote! { #[serde(rename_all = "camelCase")] }
        };
        let struct_name = &frame.struct_name;
        let struct_decl = &frame.generics.struct_decl;
        let struct_field_tokens = &frame.fields.fields;
        let struct_def = quote! {
            #derive_attr
            #serde_attr
            pub struct #struct_name #struct_decl {
                #(#struct_field_tokens),*
            }
        };
        self.code.push_str(&struct_def.to_string());
        self.code.push('\n');
    }

    fn store_class_metadata(&mut self, frame: &ClassFrame<'_>) {
        self.class_fields.insert(
            frame.meta.class_name.clone(),
            frame.fields.own_field_names.clone(),
        );
        if !frame.members.static_method_names.is_empty() {
            self.static_methods.insert(
                frame.meta.class_name.clone(),
                frame.members.static_method_names.clone(),
            );
        }
    }

    fn emit_class_impl(&mut self, frame: &ClassFrame<'_>) {
        let mut impl_items = Vec::new();
        self.emit_constructor_item(frame, &mut impl_items);
        let routes = self.emit_method_items(frame, &mut impl_items);
        self.emit_accessor_items(&frame.members, &mut impl_items);

        if frame.meta.is_controller {
            self.emit_controller_routing(
                &frame.struct_name,
                &frame.meta.class_name,
                &frame.meta.controller_info,
                &routes,
                &mut impl_items,
            );
        }

        let impl_decl = &frame.generics.impl_decl;
        let use_tokens = &frame.generics.use_tokens;
        let struct_name = &frame.struct_name;
        let impl_block = quote! {
            impl #impl_decl #struct_name #use_tokens {
                #(#impl_items)*
            }
        };
        self.code.push_str(&impl_block.to_string());
        self.code.push('\n');
    }

    fn emit_constructor_item(&mut self, frame: &ClassFrame<'_>, impl_items: &mut Vec<TokenStream>) {
        if let Some(cons) = frame.members.constructor {
            let ctx = constructor::ConstructorCtx {
                constructor: cons,
                class_fields: &frame.fields.class_fields_meta,
                has_generics: frame.decl.class.type_params.is_some(),
                generic_params: &frame.meta.generic_params,
                dependency_fields: &frame.fields.dependency_fields,
                is_service_or_controller: frame.meta.is_service_or_controller,
            };
            impl_items.push(self.convert_constructor(&frame.struct_name, &ctx));
        } else {
            impl_items.push(quote! {
                pub fn new() -> Self {
                    Self::default()
                }
                pub fn new_di() -> Self {
                    Self::default()
                }
            });
        }
    }

    fn emit_method_items(
        &mut self,
        frame: &ClassFrame<'_>,
        impl_items: &mut Vec<TokenStream>,
    ) -> Vec<(String, String, String)> {
        let mut routes = Vec::new();
        for method in &frame.members.methods {
            let (method_tokens, route_info) =
                self.convert_method(method, frame.meta.is_service_or_controller);
            impl_items.push(method_tokens);
            if let Some(info) = route_info {
                routes.push(info);
            }
        }
        routes
    }

    fn emit_accessor_items(
        &mut self,
        members: &ClassMembers<'_>,
        impl_items: &mut Vec<TokenStream>,
    ) {
        for getter in &members.getters {
            impl_items.push(self.convert_getter(getter));
        }
        for setter in &members.setters {
            impl_items.push(self.convert_setter(setter));
        }
    }
}
