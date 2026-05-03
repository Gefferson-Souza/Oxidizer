use miette::{NamedSource, SourceSpan};
use tyrus_diagnostics::TyrusError;

use swc_ecma_ast::{
    CallExpr, Callee, Expr, TsKeywordType, TsKeywordTypeKind, VarDecl, VarDeclKind,
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
}

impl Visit for LintVisitor {
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
