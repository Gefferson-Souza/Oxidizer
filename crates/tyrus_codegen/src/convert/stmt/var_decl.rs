//! Variable declaration conversion (const/let with ident, object/array destructuring).

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::{ArrayPat, BindingIdent, ObjectPat, Pat, VarDecl, VarDeclKind, VarDeclarator};

use super::super::helpers::to_snake_case;
use super::super::interface::RustGenerator;

impl RustGenerator {
    /// Convert a variable declaration statement to Rust tokens.
    /// Handles ident, object destructuring, and array destructuring patterns.
    pub(crate) fn convert_var_decl(&self, var_decl: &VarDecl) -> TokenStream {
        let mut declarations = Vec::new();
        let is_const = matches!(var_decl.kind, VarDeclKind::Const);
        for decl in &var_decl.decls {
            let init_expr_opt = decl.init.as_ref().map(|init| self.convert_expr(init));

            match &decl.name {
                Pat::Ident(ident) => {
                    self.convert_ident_decl(ident, decl, is_const, &mut declarations);
                }
                Pat::Object(obj_pat) => {
                    self.convert_object_destructuring(
                        obj_pat,
                        init_expr_opt.as_ref(),
                        &mut declarations,
                    );
                }
                Pat::Array(arr_pat) => {
                    Self::convert_array_destructuring(
                        arr_pat,
                        init_expr_opt.as_ref(),
                        &mut declarations,
                    );
                }
                _ => {
                    declarations.push(quote! { /* unsupported pattern */ });
                }
            }
        }
        quote! { #(#declarations)* }
    }

    fn convert_ident_decl(
        &self,
        ident: &BindingIdent,
        decl: &VarDeclarator,
        is_const: bool,
        declarations: &mut Vec<TokenStream>,
    ) {
        let var_name = to_snake_case(&ident.id.sym);
        let var_ident = format_ident!("{}", var_name);

        // Track typed variables for stdlib disambiguation
        if let Some(type_ann) = &ident.type_ann {
            if let swc_ecma_ast::TsType::TsKeywordType(kw) = &*type_ann.type_ann {
                if kw.kind == swc_ecma_ast::TsKeywordTypeKind::TsStringKeyword {
                    self.string_vars.borrow_mut().insert(var_name.clone());
                }
            }
            // Track Map/Set variables: Map<K,V> or Set<T>
            if let swc_ecma_ast::TsType::TsTypeRef(type_ref) = &*type_ann.type_ann {
                if let Some(ref_ident) = type_ref.type_name.as_ident() {
                    match ref_ident.sym.as_ref() {
                        "Map" => {
                            self.map_vars.borrow_mut().insert(var_name.clone());
                        }
                        "Set" => {
                            self.set_vars.borrow_mut().insert(var_name.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        // Check for typed object literal: const x: Type = { ... }
        let final_init = if let (Some(type_ann), Some(init)) = (&ident.type_ann, &decl.init) {
            if let swc_ecma_ast::Expr::Object(obj) = &**init {
                self.try_convert_typed_object_lit(&type_ann.type_ann, obj)
            } else {
                Some(self.convert_expr(init))
            }
        } else {
            decl.init.as_ref().map(|init| self.convert_expr(init))
        };

        if let Some(init_expr) = final_init {
            if is_const {
                declarations.push(quote! { let #var_ident = #init_expr; });
            } else {
                declarations.push(quote! { let mut #var_ident = #init_expr; });
            }
        } else if is_const {
            declarations.push(quote! { let #var_ident; });
        } else {
            declarations.push(quote! { let mut #var_ident; });
        }
    }

    fn convert_object_destructuring(
        &self,
        obj_pat: &ObjectPat,
        init_expr_opt: Option<&TokenStream>,
        declarations: &mut Vec<TokenStream>,
    ) {
        let Some(init_expr) = init_expr_opt else {
            return;
        };
        let source_ident = format_ident!("__destructured");
        declarations.push(quote! { let #source_ident = #init_expr; });

        for prop in &obj_pat.props {
            match prop {
                swc_ecma_ast::ObjectPatProp::KeyValue(kv) => {
                    if let swc_ecma_ast::PropName::Ident(key) = &kv.key {
                        let key_name = format_ident!("{}", to_snake_case(key.sym.as_ref()));
                        if let Pat::Ident(val_ident) = &*kv.value {
                            let val_name =
                                format_ident!("{}", to_snake_case(val_ident.sym.as_ref()));
                            declarations.push(quote! {
                                let mut #val_name = #source_ident.#key_name.clone();
                            });
                        }
                    }
                }
                swc_ecma_ast::ObjectPatProp::Assign(assign) => {
                    let key_name = format_ident!("{}", to_snake_case(assign.key.sym.as_ref()));
                    if let Some(default_val) = &assign.value {
                        let default_expr = self.convert_expr(default_val);
                        declarations.push(quote! {
                            let mut #key_name = #source_ident.#key_name.clone().unwrap_or(#default_expr);
                        });
                    } else {
                        declarations.push(quote! {
                            let mut #key_name = #source_ident.#key_name.clone();
                        });
                    }
                }
                swc_ecma_ast::ObjectPatProp::Rest(_) => {
                    declarations.push(quote! { /* rest patterns not yet supported */ });
                }
            }
        }
    }

    fn convert_array_destructuring(
        arr_pat: &ArrayPat,
        init_expr_opt: Option<&TokenStream>,
        declarations: &mut Vec<TokenStream>,
    ) {
        let Some(init_expr) = init_expr_opt else {
            return;
        };
        let source_ident = format_ident!("__arr_destructured");
        declarations.push(quote! { let #source_ident = #init_expr; });

        for (idx, elem) in arr_pat.elems.iter().enumerate() {
            if let Some(Pat::Ident(ident)) = elem {
                let var_name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                let index = idx;
                declarations.push(quote! {
                    let mut #var_name = #source_ident[#index].clone();
                });
            }
        }
    }
}
