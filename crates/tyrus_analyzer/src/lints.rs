use miette::{NamedSource, SourceSpan};
use tyrus_diagnostics::TyrusError;

use swc_common::Spanned;
use swc_ecma_ast::{
    CallExpr, Callee, Decl, DefaultDecl, Expr, FnDecl, Module, ModuleDecl, ModuleItem, Script,
    Stmt, TsKeywordType, TsKeywordTypeKind, VarDecl, VarDeclKind,
};
use swc_ecma_visit::{Visit, VisitWith};

pub(crate) struct LintVisitor {
    pub(crate) errors: Vec<TyrusError>,
    pub(crate) source_code: String,
    pub(crate) file_name: String,
}

impl LintVisitor {
    pub(crate) fn new(source_code: String, file_name: String) -> Self {
        Self {
            errors: Vec::new(),
            source_code,
            file_name,
        }
    }

    fn create_span(&self, span: swc_common::Span) -> SourceSpan {
        let start = span.lo.0 as usize - 1;
        let end = span.hi.0 as usize - 1;
        let len = end - start;
        SourceSpan::new(start.into(), len)
    }

    /// D1 — emit `AmbiguousMainEntrypoint` when the file both declares a
    /// user-level `function main()` AND contains top-level non-decl statements.
    /// Either alone is fine; combined, the codegen at `tyrus_codegen::generate`
    /// (lib.rs `!has_declared_main` gate) would either drop the statements or
    /// duplicate `fn main()`. Mirrors codegen's `has_declared_main` detection
    /// across all three TS forms: bare, `export`, and `export default`.
    fn check_ambiguous_main(&mut self, items: &ModuleItemsView<'_>) {
        let (has_user_main, first_top_stmt_span) = items.classify();
        if let (true, Some(span)) = (has_user_main, first_top_stmt_span) {
            self.errors.push(TyrusError::AmbiguousMainEntrypoint {
                src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
                span: self.create_span(span),
            });
        }
    }
}

/// Wraps either `&Module` or `&Script` items so D1 can scan a single uniform
/// API. `Module` items may carry an `export function main()` decl (under
/// `ModuleDecl::ExportDecl` or `ModuleDecl::ExportDefaultDecl`) — codegen
/// recognises both, so D1 must mirror that.
enum ModuleItemsView<'a> {
    Module(&'a [ModuleItem]),
    Script(&'a [Stmt]),
}

impl ModuleItemsView<'_> {
    fn classify(&self) -> (bool, Option<swc_common::Span>) {
        let mut has_user_main = false;
        let mut first_top_stmt_span: Option<swc_common::Span> = None;
        match self {
            ModuleItemsView::Module(items) => {
                for item in *items {
                    match item {
                        ModuleItem::Stmt(stmt) => {
                            classify_stmt(stmt, &mut has_user_main, &mut first_top_stmt_span);
                        }
                        ModuleItem::ModuleDecl(decl) => {
                            classify_module_decl(decl, &mut has_user_main);
                        }
                    }
                }
            }
            ModuleItemsView::Script(stmts) => {
                for stmt in *stmts {
                    classify_stmt(stmt, &mut has_user_main, &mut first_top_stmt_span);
                }
            }
        }
        (has_user_main, first_top_stmt_span)
    }
}

fn classify_stmt(
    stmt: &Stmt,
    has_user_main: &mut bool,
    first_top_stmt_span: &mut Option<swc_common::Span>,
) {
    match stmt {
        Stmt::Decl(Decl::Fn(fn_decl)) if is_main_fn(fn_decl) => {
            *has_user_main = true;
        }
        Stmt::Decl(_) => {}
        other => {
            if first_top_stmt_span.is_none() {
                *first_top_stmt_span = Some(other.span());
            }
        }
    }
}

fn classify_module_decl(decl: &ModuleDecl, has_user_main: &mut bool) {
    match decl {
        ModuleDecl::ExportDecl(export) => {
            if let Decl::Fn(fn_decl) = &export.decl {
                if is_main_fn(fn_decl) {
                    *has_user_main = true;
                }
            }
        }
        ModuleDecl::ExportDefaultDecl(default) => {
            if let DefaultDecl::Fn(fn_expr) = &default.decl {
                if let Some(ident) = &fn_expr.ident {
                    if ident.sym == "main" {
                        *has_user_main = true;
                    }
                }
            }
        }
        _ => {}
    }
}

fn is_main_fn(fn_decl: &FnDecl) -> bool {
    fn_decl.ident.sym == "main"
}

impl Visit for LintVisitor {
    fn visit_module(&mut self, n: &Module) {
        self.check_ambiguous_main(&ModuleItemsView::Module(&n.body));
        n.visit_children_with(self);
    }

    fn visit_script(&mut self, n: &Script) {
        self.check_ambiguous_main(&ModuleItemsView::Script(&n.body));
        n.visit_children_with(self);
    }

    fn visit_var_decl(&mut self, n: &VarDecl) {
        if n.kind == VarDeclKind::Var {
            self.errors.push(TyrusError::UseOfVar {
                src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
                span: self.create_span(n.span),
            });
        }
        n.visit_children_with(self);
    }

    fn visit_ts_keyword_type(&mut self, n: &TsKeywordType) {
        if n.kind == TsKeywordTypeKind::TsAnyKeyword {
            self.errors.push(TyrusError::UseOfAny {
                src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
                span: self.create_span(n.span),
            });
        }
        n.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, n: &CallExpr) {
        if let Callee::Expr(expr) = &n.callee {
            if let Expr::Ident(ident) = &**expr {
                if ident.sym == "eval" {
                    self.errors.push(TyrusError::UseOfEval {
                        src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
                        span: self.create_span(n.span),
                    });
                }
            }
        }
        n.visit_children_with(self);
    }

    // Supported: while, do-while, for, for-of, switch, try-catch
    // Blocked: for-in (codegen not yet implemented)

    fn visit_for_in_stmt(&mut self, n: &swc_ecma_ast::ForInStmt) {
        self.errors.push(TyrusError::UnsupportedFeature {
            feature: "for-in loops".to_string(),
            src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
            span: self.create_span(n.span),
        });
        n.visit_children_with(self);
    }

    fn visit_unary_expr(&mut self, n: &swc_ecma_ast::UnaryExpr) {
        if n.op == swc_ecma_ast::UnaryOp::Delete {
            self.errors.push(TyrusError::UnsupportedFeature {
                feature: "delete operator".to_string(),
                src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
                span: self.create_span(n.span),
            });
        }
        n.visit_children_with(self);
    }

    fn visit_with_stmt(&mut self, n: &swc_ecma_ast::WithStmt) {
        self.errors.push(TyrusError::UnsupportedFeature {
            feature: "with statement".to_string(),
            src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
            span: self.create_span(n.span),
        });
        n.visit_children_with(self);
    }

    fn visit_labeled_stmt(&mut self, n: &swc_ecma_ast::LabeledStmt) {
        self.errors.push(TyrusError::UnsupportedFeature {
            feature: "labeled statements".to_string(),
            src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
            span: self.create_span(n.span),
        });
        n.visit_children_with(self);
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use swc_common::sync::Lrc;
    use swc_common::{FileName, SourceMap};
    use swc_ecma_ast::Program;
    use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax};

    fn parse_module(source: &str) -> Module {
        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Anon.into(), source.to_string());
        let mut parser = Parser::new(
            Syntax::Typescript(TsSyntax::default()),
            StringInput::from(&*fm),
            None,
        );
        parser.parse_module().expect("parse_module")
    }

    fn run_lints(source: &str) -> Vec<TyrusError> {
        let module = parse_module(source);
        let program = Program::Module(module);
        let mut visitor = LintVisitor::new(source.to_string(), "test.ts".to_string());
        program.visit_with(&mut visitor);
        visitor.errors
    }

    #[test]
    fn d1_reports_when_user_main_and_top_level_stmt_coexist() {
        let errors = run_lints(
            r#"
function main(): void {
    console.log("user main");
}
console.log("orphan top level");
"#,
        );
        let has_d1 = errors
            .iter()
            .any(|e| matches!(e, TyrusError::AmbiguousMainEntrypoint { .. }));
        assert!(has_d1, "expected AmbiguousMainEntrypoint, got: {errors:?}");
    }

    #[test]
    fn d1_silent_when_only_user_main() {
        let errors = run_lints(
            r#"
function main(): void {
    console.log("just main");
}
"#,
        );
        let has_d1 = errors
            .iter()
            .any(|e| matches!(e, TyrusError::AmbiguousMainEntrypoint { .. }));
        assert!(!has_d1, "no diagnostic expected, got: {errors:?}");
    }

    #[test]
    fn d1_silent_when_only_top_level_stmts() {
        let errors = run_lints(
            r#"
console.log("hello");
"#,
        );
        let has_d1 = errors
            .iter()
            .any(|e| matches!(e, TyrusError::AmbiguousMainEntrypoint { .. }));
        assert!(!has_d1, "no diagnostic expected, got: {errors:?}");
    }

    #[test]
    fn d1_silent_when_only_decls() {
        let errors = run_lints(
            r#"
interface User { name: string; }
function helper(): number { return 42; }
function main(): void { console.log("ok"); }
"#,
        );
        let has_d1 = errors
            .iter()
            .any(|e| matches!(e, TyrusError::AmbiguousMainEntrypoint { .. }));
        assert!(!has_d1, "no diagnostic expected, got: {errors:?}");
    }

    #[test]
    fn d1_reports_when_exported_main_and_top_level_stmt_coexist() {
        let errors = run_lints(
            r#"
export function main(): void {
    console.log("exported main");
}
console.log("orphan");
"#,
        );
        let has_d1 = errors
            .iter()
            .any(|e| matches!(e, TyrusError::AmbiguousMainEntrypoint { .. }));
        assert!(
            has_d1,
            "expected AmbiguousMainEntrypoint for exported main, got: {errors:?}"
        );
    }

    #[test]
    fn d1_reports_when_export_default_main_and_top_level_stmt_coexist() {
        let errors = run_lints(
            r#"
export default function main(): void {
    console.log("default main");
}
console.log("orphan");
"#,
        );
        let has_d1 = errors
            .iter()
            .any(|e| matches!(e, TyrusError::AmbiguousMainEntrypoint { .. }));
        assert!(
            has_d1,
            "expected AmbiguousMainEntrypoint for export default main, got: {errors:?}"
        );
    }
}
