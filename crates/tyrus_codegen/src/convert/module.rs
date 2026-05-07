use swc_ecma_ast::{ImportSpecifier, ModuleDecl, ModuleItem};
use swc_ecma_visit::{Visit, VisitWith};

use super::interface::RustGenerator;

/// Convert a TS name to its Rust equivalent.
/// Uppercase-leading names are preserved (Class/Type);
/// lowercase-leading names are converted to snake_case.
fn to_rust_name(name: &str) -> String {
    if name.chars().next().is_some_and(char::is_uppercase) {
        name.to_string()
    } else {
        super::helpers::to_snake_case(name)
    }
}

/// Replace `.` and `-` in each segment with `_`, then join with `::`.
fn sanitize_path(p: &str) -> String {
    p.split('/')
        .map(|part| part.replace(['.', '-'], "_"))
        .collect::<Vec<_>>()
        .join("::")
}

/// Return `true` if this import source should be silently skipped
/// (NestJS framework imports, axios HTTP client, etc.).
fn is_skipped_import(src: &str) -> bool {
    src.starts_with("@nestjs") || src == "axios"
}

/// Resolve a TS import path to a Rust module path.
/// Relative paths (`./ ../`) become `self::` / `super::` depending on
/// whether the current file is an index module.
fn resolve_module_path(src: &str, is_index: bool) -> String {
    if let Some(rest) = src.strip_prefix("./") {
        let sanitized = sanitize_path(rest);
        if is_index {
            format!("self::{sanitized}")
        } else {
            format!("super::{sanitized}")
        }
    } else if let Some(rest) = src.strip_prefix("../") {
        let sanitized = sanitize_path(rest);
        if is_index {
            format!("super::{sanitized}")
        } else {
            format!("super::super::{sanitized}")
        }
    } else {
        src.to_string()
    }
}

impl RustGenerator {
    // ------------------------------------------------------------------
    // Module items
    // ------------------------------------------------------------------

    pub(crate) fn process_module_item(&mut self, n: &ModuleItem) {
        match n {
            ModuleItem::ModuleDecl(decl) => self.process_module_decl(decl),
            // `visit_with` traverses children only — bypassing the custom
            // `visit_stmt` override at `interface.rs`. Direct dispatch ensures
            // top-level statements route into `main_body` instead of being dropped.
            ModuleItem::Stmt(stmt) => self.visit_stmt(stmt),
        }
    }

    fn process_module_decl(&mut self, decl: &ModuleDecl) {
        match decl {
            ModuleDecl::ExportDecl(export_decl) => {
                self.is_exporting = true;
                export_decl.decl.visit_with(self);
                self.is_exporting = false;
            }
            ModuleDecl::ExportDefaultDecl(default_decl) => {
                self.process_export_default(&default_decl.decl);
            }
            ModuleDecl::Import(import_decl) => {
                self.process_import_decl(import_decl);
            }
            _ => {
                // Other module declarations (re-exports, etc.)
            }
        }
    }

    fn process_export_default(&mut self, decl: &swc_ecma_ast::DefaultDecl) {
        self.is_exporting = true;
        match decl {
            swc_ecma_ast::DefaultDecl::Class(class_expr) => {
                if let Some(ident) = &class_expr.ident {
                    let class_decl = swc_ecma_ast::ClassDecl {
                        ident: ident.clone(),
                        declare: false,
                        class: class_expr.class.clone(),
                    };
                    self.process_class_decl(&class_decl);
                }
            }
            swc_ecma_ast::DefaultDecl::Fn(fn_expr) => {
                if let Some(ident) = &fn_expr.ident {
                    let fn_decl = swc_ecma_ast::FnDecl {
                        ident: ident.clone(),
                        declare: false,
                        function: fn_expr.function.clone(),
                    };
                    self.process_fn_decl(&fn_decl);
                }
            }
            _ => {}
        }
        self.is_exporting = false;
    }

    // ------------------------------------------------------------------
    // Imports
    // ------------------------------------------------------------------

    fn process_import_decl(&mut self, n: &swc_ecma_ast::ImportDecl) {
        let src = Self::normalize_import_source(n);

        if is_skipped_import(&src) {
            return;
        }

        let module_path = resolve_module_path(&src, self.is_index);

        for specifier in &n.specifiers {
            let use_stmt = format_use_stmt(specifier, &module_path);
            self.code.push_str(&use_stmt);
            self.code.push('\n');
        }
    }

    /// Extract and normalize the import source string
    /// (strip `/index` suffix, convert to owned `String`).
    fn normalize_import_source(n: &swc_ecma_ast::ImportDecl) -> String {
        let raw = n.src.value.as_str().unwrap_or("");
        let stripped = raw.strip_suffix("/index").unwrap_or(raw);
        stripped.to_string()
    }
}

// ------------------------------------------------------------------
// Free helpers for formatting use-statements
// ------------------------------------------------------------------

/// Build a `use …;` string for a single import specifier.
fn format_use_stmt(specifier: &ImportSpecifier, module_path: &str) -> String {
    match specifier {
        ImportSpecifier::Named(named) => format_named_use(named, module_path),
        ImportSpecifier::Default(default) => {
            let local = to_rust_name(&default.local.sym);
            format!("use {module_path}::{local};")
        }
        ImportSpecifier::Namespace(ns) => {
            let local = to_rust_name(&ns.local.sym);
            format!("use {module_path} as {local};")
        }
    }
}

/// Build a `use …;` for a named specifier, handling optional aliasing.
fn format_named_use(named: &swc_ecma_ast::ImportNamedSpecifier, module_path: &str) -> String {
    let imported_name = extract_imported_name(named);
    let imported_rust = to_rust_name(&imported_name);
    let local_rust = to_rust_name(&named.local.sym);

    if imported_rust == local_rust {
        format!("use {module_path}::{local_rust};")
    } else {
        format!("use {module_path}::{imported_rust} as {local_rust};")
    }
}

/// Return the original imported identifier string.
/// Falls back to the local name when there is no explicit `imported` alias.
fn extract_imported_name(named: &swc_ecma_ast::ImportNamedSpecifier) -> String {
    match &named.imported {
        Some(swc_ecma_ast::ModuleExportName::Ident(ident)) => ident.sym.to_string(),
        Some(swc_ecma_ast::ModuleExportName::Str(s)) => s.value.as_str().unwrap_or("").to_string(),
        None => named.local.sym.to_string(),
    }
}
