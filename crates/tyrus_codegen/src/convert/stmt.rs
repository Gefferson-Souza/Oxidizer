//! Statement-level code generation.
//!
//! Converts TypeScript statements (variable declarations, if/else, while, for-of,
//! return, throw, try-catch, switch, do-while) to Rust TokenStreams.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use swc_ecma_ast::*;

use super::helpers::to_snake_case;
use super::interface::RustGenerator;

impl RustGenerator {
    /// Recursively convert statements with a custom return handler.
    /// Used by process_fn_decl to inject Result wrapping for async functions.
    pub fn convert_stmt_recursive<F>(&self, stmt: &Stmt, return_handler: &mut F) -> TokenStream
    where
        F: FnMut(&swc_ecma_ast::ReturnStmt) -> TokenStream,
    {
        match stmt {
            Stmt::Return(ret_stmt) => return_handler(ret_stmt),
            Stmt::Block(block) => {
                let stmts: Vec<_> = block
                    .stmts
                    .iter()
                    .map(|s| self.convert_stmt_recursive(s, return_handler))
                    .collect();
                quote! { { #(#stmts)* } }
            }
            Stmt::If(if_stmt) => {
                let test = self.convert_expr(&if_stmt.test);
                let cons = self.convert_stmt_recursive(&if_stmt.cons, return_handler);
                let cons_block = if matches!(*if_stmt.cons, Stmt::Block(_)) {
                    quote! { #cons }
                } else {
                    quote! { { #cons } }
                };

                let alt = if let Some(alt) = &if_stmt.alt {
                    let alt_stmt = self.convert_stmt_recursive(alt, return_handler);
                    let alt_block = if matches!(&**alt, Stmt::Block(_) | Stmt::If(_)) {
                        quote! { #alt_stmt }
                    } else {
                        quote! { { #alt_stmt } }
                    };
                    quote! { else #alt_block }
                } else {
                    quote! {}
                };

                quote! { if #test #cons_block #alt }
            }
            _ => self.convert_stmt(stmt),
        }
    }

    /// Convert a single TypeScript statement to Rust tokens.
    pub fn convert_stmt(&self, stmt: &Stmt) -> TokenStream {
        match stmt {
            Stmt::Return(ret_stmt) => {
                if let Some(arg) = &ret_stmt.arg {
                    let expr = self.convert_expr(arg);
                    quote! { return #expr; }
                } else {
                    quote! { return; }
                }
            }
            Stmt::Expr(expr_stmt) => {
                let expr = self.convert_expr(&expr_stmt.expr);
                quote! { #expr; }
            }
            Stmt::Decl(Decl::Var(var_decl)) => {
                let mut declarations = Vec::new();
                let is_const = matches!(var_decl.kind, VarDeclKind::Const);
                for decl in &var_decl.decls {
                    // Pre-compute init expression for non-Ident patterns
                    let init_expr_opt = decl.init.as_ref().map(|init| self.convert_expr(init));

                    match &decl.name {
                        Pat::Ident(ident) => {
                            let var_name = to_snake_case(&ident.id.sym);
                            let var_ident = format_ident!("{}", var_name);

                            // Track string-typed variables for stdlib disambiguation
                            if let Some(type_ann) = &ident.type_ann {
                                if let swc_ecma_ast::TsType::TsKeywordType(kw) = &*type_ann.type_ann
                                {
                                    if kw.kind == swc_ecma_ast::TsKeywordTypeKind::TsStringKeyword {
                                        self.string_vars.borrow_mut().insert(var_name.clone());
                                    }
                                }
                            }

                            // Check for typed object literal: const x: Type = { ... }
                            let final_init = if let (Some(type_ann), Some(init)) =
                                (&ident.type_ann, &decl.init)
                            {
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
                                    declarations.push(quote! {
                                        let #var_ident = #init_expr;
                                    });
                                } else {
                                    declarations.push(quote! {
                                        let mut #var_ident = #init_expr;
                                    });
                                }
                            } else if is_const {
                                declarations.push(quote! {
                                    let #var_ident;
                                });
                            } else {
                                declarations.push(quote! {
                                    let mut #var_ident;
                                });
                            }
                        }
                        Pat::Object(obj_pat) => {
                            // Object destructuring: const { id, name = "default" } = expr
                            if let Some(init_expr) = &init_expr_opt {
                                let source_ident = format_ident!("__destructured");
                                declarations.push(quote! {
                                    let #source_ident = #init_expr;
                                });
                                for prop in &obj_pat.props {
                                    match prop {
                                        swc_ecma_ast::ObjectPatProp::KeyValue(kv) => {
                                            if let swc_ecma_ast::PropName::Ident(key) = &kv.key {
                                                let key_name = format_ident!(
                                                    "{}",
                                                    to_snake_case(key.sym.as_ref())
                                                );
                                                if let Pat::Ident(val_ident) = &*kv.value {
                                                    let val_name = format_ident!(
                                                        "{}",
                                                        to_snake_case(val_ident.sym.as_ref())
                                                    );
                                                    declarations.push(quote! {
                                                        let mut #val_name = #source_ident.#key_name.clone();
                                                    });
                                                }
                                            }
                                        }
                                        swc_ecma_ast::ObjectPatProp::Assign(assign) => {
                                            let key_name = format_ident!(
                                                "{}",
                                                to_snake_case(assign.key.sym.as_ref())
                                            );
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
                                            declarations.push(
                                                quote! { /* rest patterns not yet supported */ },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        Pat::Array(arr_pat) => {
                            // Array destructuring: const [first, second] = list
                            if let Some(init_expr) = &init_expr_opt {
                                let source_ident = format_ident!("__arr_destructured");
                                declarations.push(quote! {
                                    let #source_ident = #init_expr;
                                });
                                for (idx, elem) in arr_pat.elems.iter().enumerate() {
                                    if let Some(Pat::Ident(ident)) = elem {
                                        let var_name =
                                            format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                                        let index = idx;
                                        declarations.push(quote! {
                                            let mut #var_name = #source_ident[#index].clone();
                                        });
                                    }
                                }
                            }
                        }
                        _ => {
                            declarations.push(quote! { /* unsupported pattern */ });
                        }
                    }
                }
                quote! { #(#declarations)* }
            }
            Stmt::Block(block) => {
                let stmts: Vec<_> = block.stmts.iter().map(|s| self.convert_stmt(s)).collect();
                quote! { { #(#stmts)* } }
            }
            Stmt::If(if_stmt) => {
                let test = self.convert_expr(&if_stmt.test);
                let cons = self.convert_stmt(&if_stmt.cons);
                let cons_block = if matches!(*if_stmt.cons, Stmt::Block(_)) {
                    quote! { #cons }
                } else {
                    quote! { { #cons } }
                };

                let alt = if let Some(alt) = &if_stmt.alt {
                    let alt_stmt = self.convert_stmt(alt);
                    let alt_block = if matches!(&**alt, Stmt::Block(_) | Stmt::If(_)) {
                        quote! { #alt_stmt }
                    } else {
                        quote! { { #alt_stmt } }
                    };
                    quote! { else #alt_block }
                } else {
                    quote! {}
                };

                quote! { if #test #cons_block #alt }
            }
            Stmt::While(while_stmt) => {
                let test = self.convert_expr(&while_stmt.test);
                let body = self.convert_stmt(&while_stmt.body);
                let body_block = if matches!(*while_stmt.body, Stmt::Block(_)) {
                    quote! { #body }
                } else {
                    quote! { { #body } }
                };
                quote! {
                    while #test #body_block
                }
            }
            Stmt::ForOf(for_of) => {
                let body = self.convert_stmt(&for_of.body);
                let right = self.convert_expr(&for_of.right);
                let body_block = if matches!(*for_of.body, Stmt::Block(_)) {
                    quote! { #body }
                } else {
                    quote! { { #body } }
                };

                // Extract the loop variable name from the declaration
                let var_ident = match &for_of.left {
                    swc_ecma_ast::ForHead::VarDecl(var_decl) => {
                        if let Some(decl) = var_decl.decls.first() {
                            if let Pat::Ident(ident) = &decl.name {
                                let name = to_snake_case(&ident.id.sym);
                                format_ident!("{}", name)
                            } else {
                                format_ident!("_item")
                            }
                        } else {
                            format_ident!("_item")
                        }
                    }
                    swc_ecma_ast::ForHead::Pat(pat) => {
                        if let Pat::Ident(ident) = pat.as_ref() {
                            let name = to_snake_case(&ident.id.sym);
                            format_ident!("{}", name)
                        } else {
                            format_ident!("_item")
                        }
                    }
                    _ => format_ident!("_item"),
                };

                quote! {
                    for #var_ident in #right #body_block
                }
            }
            Stmt::ForIn(for_in) => {
                let body = self.convert_stmt(&for_in.body);
                let right = self.convert_expr(&for_in.right);
                let body_block = if matches!(*for_in.body, Stmt::Block(_)) {
                    quote! { #body }
                } else {
                    quote! { { #body } }
                };

                let var_ident = match &for_in.left {
                    swc_ecma_ast::ForHead::VarDecl(var_decl) => {
                        if let Some(decl) = var_decl.decls.first() {
                            if let Pat::Ident(ident) = &decl.name {
                                format_ident!("{}", to_snake_case(&ident.id.sym))
                            } else {
                                format_ident!("_key")
                            }
                        } else {
                            format_ident!("_key")
                        }
                    }
                    swc_ecma_ast::ForHead::Pat(pat) => {
                        if let Pat::Ident(ident) = pat.as_ref() {
                            format_ident!("{}", to_snake_case(&ident.id.sym))
                        } else {
                            format_ident!("_key")
                        }
                    }
                    _ => format_ident!("_key"),
                };

                quote! {
                    for #var_ident in #right.keys() #body_block
                }
            }
            Stmt::For(for_stmt) => {
                let init = match &for_stmt.init {
                    Some(swc_ecma_ast::VarDeclOrExpr::VarDecl(var_decl)) => {
                        self.convert_stmt(&Stmt::Decl(Decl::Var(var_decl.clone())))
                    }
                    Some(swc_ecma_ast::VarDeclOrExpr::Expr(expr)) => {
                        let e = self.convert_expr(expr);
                        quote! { #e; }
                    }
                    None => quote! {},
                };
                let test = for_stmt
                    .test
                    .as_ref()
                    .map(|t| self.convert_expr(t))
                    .unwrap_or_else(|| quote! { true });
                let update = for_stmt
                    .update
                    .as_ref()
                    .map(|u| {
                        let expr = self.convert_expr(u);
                        quote! { #expr; }
                    })
                    .unwrap_or_default();
                let body = self.convert_stmt(&for_stmt.body);
                let body_block = if matches!(*for_stmt.body, Stmt::Block(_)) {
                    body
                } else {
                    quote! { { #body } }
                };

                // for(init; test; update) { body }
                // → { init; while test { body; update; } }
                quote! {
                    {
                        #init
                        while #test {
                            #body_block
                            #update
                        }
                    }
                }
            }
            Stmt::DoWhile(do_while) => {
                let test = self.convert_expr(&do_while.test);
                let body = self.convert_stmt(&do_while.body);
                let body_block = if matches!(*do_while.body, Stmt::Block(_)) {
                    body
                } else {
                    quote! { { #body } }
                };

                // do { body } while (cond) → loop { body; if !cond { break; } }
                quote! {
                    loop {
                        #body_block
                        if !(#test) { break; }
                    }
                }
            }
            Stmt::Switch(switch_stmt) => self.convert_switch_stmt(switch_stmt),
            Stmt::Throw(throw_stmt) => {
                let arg = self.convert_expr(&throw_stmt.arg);
                quote! {
                    return Err(#arg.into());
                }
            }
            _ => quote! { /* other stmts todo */ },
        }
    }

    /// Convert a switch statement to a Rust match expression.
    /// Each case with a body becomes a match arm. Cases with only `return`
    /// are straightforward; fall-through is not supported.
    fn convert_switch_stmt(&self, switch: &swc_ecma_ast::SwitchStmt) -> TokenStream {
        let discriminant = self.convert_expr(&switch.discriminant);
        let mut arms = Vec::new();

        for case in &switch.cases {
            let body: Vec<_> = case
                .cons
                .iter()
                .filter(|s| !matches!(s, Stmt::Break(_)))
                .map(|s| self.convert_stmt(s))
                .collect();

            if body.is_empty() {
                continue;
            }

            if let Some(test) = &case.test {
                let test_expr = self.convert_expr(test);
                arms.push(quote! { __v if __v == #test_expr => { #(#body)* } });
            } else {
                // default case
                arms.push(quote! { _ => { #(#body)* } });
            }
        }

        // Ensure there's always a default arm
        let has_default = switch.cases.iter().any(|c| c.test.is_none());
        if !has_default {
            arms.push(quote! { _ => {} });
        }

        quote! {
            match #discriminant {
                #(#arms)*
            }
        }
    }
}
