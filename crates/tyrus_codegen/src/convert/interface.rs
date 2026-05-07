use quote::{format_ident, quote};
use swc_ecma_ast::{TsInterfaceDecl, TsTypeElement};
use swc_ecma_visit::{Visit, VisitWith};

use super::type_decl;
use super::type_mapper::map_ts_type;

use crate::ControllerMetadata;

#[derive(Default)]
pub struct RustGenerator {
    pub code: String,
    pub is_exporting: bool,
    pub is_index: bool,
    pub is_controller: bool,
    pub controllers: Vec<ControllerMetadata>,
    pub main_body: String,
    pub has_declared_main: bool,
    pub current_class_state_fields: std::collections::HashMap<String, String>,
    /// Class fields for inheritance (parent fields flattened into child)
    pub(crate) class_fields:
        std::collections::HashMap<String, Vec<(String, proc_macro2::TokenStream, bool)>>,
    /// Static methods per class (ClassName → method_names)
    pub(crate) static_methods: std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// When true, Expr::This generates `state` instead of `self`
    pub(crate) use_state_for_this: std::cell::Cell<bool>,
    /// Variables with `: string` annotation (stdlib disambiguation)
    pub(crate) string_vars: std::cell::RefCell<std::collections::HashSet<String>>,
    /// Getter property names (call-site: obj.prop → obj.prop())
    pub(crate) getter_names: std::collections::HashSet<String>,
    /// Setter property names (call-site: obj.prop = v → obj.set_prop(v))
    pub(crate) setter_names: std::collections::HashSet<String>,
    /// Variables with Map type annotation (stdlib disambiguation)
    pub(crate) map_vars: std::cell::RefCell<std::collections::HashSet<String>>,
    /// Variables with Set type annotation (stdlib disambiguation)
    pub(crate) set_vars: std::cell::RefCell<std::collections::HashSet<String>>,
}

impl RustGenerator {
    pub fn new(is_index: bool) -> Self {
        Self {
            code: String::new(),
            is_exporting: false,
            is_index,
            is_controller: false,
            controllers: Vec::new(),
            main_body: String::new(),
            has_declared_main: false,
            current_class_state_fields: std::collections::HashMap::new(),
            class_fields: std::collections::HashMap::new(),
            static_methods: std::collections::HashMap::new(),
            use_state_for_this: std::cell::Cell::new(false),
            string_vars: std::cell::RefCell::new(std::collections::HashSet::new()),
            getter_names: std::collections::HashSet::new(),
            setter_names: std::collections::HashSet::new(),
            map_vars: std::cell::RefCell::new(std::collections::HashSet::new()),
            set_vars: std::cell::RefCell::new(std::collections::HashSet::new()),
        }
    }
}

fn convert_interface_fields(body: &[TsTypeElement]) -> Vec<proc_macro2::TokenStream> {
    body.iter()
        .filter_map(|member| {
            if let TsTypeElement::TsPropertySignature(prop) = member {
                let name_str = prop.key.as_ident()?.sym.to_string();
                let field_name = format_ident!("{}", super::helpers::to_snake_case(&name_str));
                let mut field_type = map_ts_type(prop.type_ann.as_ref());
                if prop.optional {
                    field_type = quote! { Option<#field_type> };
                }
                Some(quote! { pub #field_name: #field_type })
            } else {
                None
            }
        })
        .collect()
}

fn convert_interface_generics(
    type_params: Option<&swc_ecma_ast::TsTypeParamDecl>,
) -> proc_macro2::TokenStream {
    let Some(params) = type_params else {
        return quote! {};
    };
    let idents: Vec<_> = params
        .params
        .iter()
        .map(|p| {
            let ident = format_ident!("{}", p.name.sym.to_string());
            quote! { #ident: Clone }
        })
        .collect();
    quote! { <#(#idents),*> }
}

impl Visit for RustGenerator {
    fn visit_ts_interface_decl(&mut self, n: &TsInterfaceDecl) {
        let struct_name = format_ident!("{}", n.id.sym.to_string());
        let fields = convert_interface_fields(&n.body.body);
        let generics = convert_interface_generics(n.type_params.as_deref());

        let struct_def = quote! {
            #[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct #struct_name #generics {
                #(#fields),*
            }
        };

        self.code.push_str(&struct_def.to_string());
        self.code.push('\n');
    }

    fn visit_fn_decl(&mut self, n: &swc_ecma_ast::FnDecl) {
        self.process_fn_decl(n);
    }

    fn visit_class_decl(&mut self, n: &swc_ecma_ast::ClassDecl) {
        self.process_class_decl(n);
    }

    fn visit_ts_type_alias_decl(&mut self, n: &swc_ecma_ast::TsTypeAliasDecl) {
        let tokens = type_decl::convert_type_alias(n, self.is_exporting);
        self.code.push_str(&tokens.to_string());
        self.code.push('\n');
    }

    fn visit_ts_enum_decl(&mut self, n: &swc_ecma_ast::TsEnumDecl) {
        let tokens = type_decl::convert_enum(n, self.is_exporting);
        self.code.push_str(&tokens.to_string());
        self.code.push('\n');
    }

    fn visit_module_item(&mut self, n: &swc_ecma_ast::ModuleItem) {
        self.process_module_item(n);
    }

    fn visit_stmt(&mut self, n: &swc_ecma_ast::Stmt) {
        // This is called for top-level statements via process_module_item -> visit_with(self)
        match n {
            swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Fn(_))
            | swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Class(_))
            | swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::TsInterface(_))
            | swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::TsTypeAlias(_))
            | swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::TsEnum(_)) => {
                // Top-level declarations: let visitor handle them (writes to self.code)
                n.visit_children_with(self);
            }
            _ => {
                // Script statements (ExprStmt, VarDecl, If, Loop, etc.): write to self.main_body
                let stmt_code = self.convert_stmt(n);
                self.main_body.push_str(&stmt_code.to_string());
                self.main_body.push('\n');
            }
        }
    }
}
