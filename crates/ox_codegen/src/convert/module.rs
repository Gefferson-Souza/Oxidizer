use swc_ecma_ast::{ModuleDecl, ModuleItem};
use swc_ecma_visit::VisitWith;

use super::interface::RustGenerator;

impl RustGenerator {
    pub fn process_module_item(&mut self, n: &ModuleItem) {
        match n {
            ModuleItem::ModuleDecl(decl) => match decl {
                ModuleDecl::ExportDecl(export_decl) => {
                    self.is_exporting = true;
                    export_decl.decl.visit_with(self);
                    self.is_exporting = false;
                }
                ModuleDecl::Import(import_decl) => {
                    self.process_import_decl(import_decl);
                }
                _ => {
                    // Other module declarations
                }
            },
            ModuleItem::Stmt(stmt) => {
                stmt.visit_with(self);
            }
        }
    }

    fn process_import_decl(&mut self, n: &swc_ecma_ast::ImportDecl) {
        let src_atom = &n.src.value;
        let src = src_atom.as_str().unwrap_or("");

        // Simple path resolution: ./foo -> crate::foo
        // This is a naive implementation and assumes a flat structure or simple relative imports
        let module_path = if src.starts_with("./") {
            let path_str = src.trim_start_matches("./");
            format!("crate::{}", path_str.replace("/", "::"))
        } else {
            // External crate or absolute path - keep as is or map known libs
            src.to_string()
        };

        for specifier in &n.specifiers {
            match specifier {
                swc_ecma_ast::ImportSpecifier::Named(named) => {
                    let imported_name = if let Some(imported) = &named.imported {
                        match imported {
                            swc_ecma_ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
                            swc_ecma_ast::ModuleExportName::Str(s) => {
                                s.value.as_str().unwrap_or("").to_string()
                            }
                        }
                    } else {
                        named.local.sym.to_string()
                    };
                    let local_name = named.local.sym.to_string();

                    let use_stmt = if imported_name == local_name {
                        format!("use {}::{};", module_path, local_name)
                    } else {
                        format!("use {}::{} as {};", module_path, imported_name, local_name)
                    };

                    self.code.push_str(&use_stmt);
                    self.code.push('\n');
                }
                swc_ecma_ast::ImportSpecifier::Default(default) => {
                    // Default import: import Foo from './bar' -> use crate::bar::Foo; (assuming default export is named same as file or struct)
                    // This is tricky in Rust. For now, let's assume the module exposes a struct/fn with the same name as the local binding
                    let local_name = default.local.sym.to_string();
                    let use_stmt = format!("use {}::{};", module_path, local_name);
                    self.code.push_str(&use_stmt);
                    self.code.push('\n');
                }
                swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
                    // import * as foo from './bar' -> use crate::bar as foo;
                    let local_name = ns.local.sym.to_string();
                    let use_stmt = format!("use {} as {};", module_path, local_name);
                    self.code.push_str(&use_stmt);
                    self.code.push('\n');
                }
            }
        }
    }
}
