// Visitor modules are crate-private — only `Analyzer::analyze` should
// instantiate them. `report` and `severity` are exposed because the CLI
// and orchestrator format diagnostics for end users.
pub(crate) mod decorators;
pub(crate) mod lints;
pub mod report;
pub mod severity;
pub(crate) mod unsupported;

use crate::decorators::DecoratorVisitor;
use crate::lints::LintVisitor;
use crate::unsupported::UnsupportedApiVisitor;
use swc_ecma_ast::Program;
use swc_ecma_visit::VisitWith;
use tyrus_di::graph::DiGraph;
use tyrus_diagnostics::TyrusError;

pub struct Analyzer;

pub struct AnalysisResult {
    pub errors: Vec<TyrusError>,
    pub diagnostics: Vec<severity::Diagnostic>,
    pub graph: DiGraph,
}

impl Analyzer {
    pub fn analyze(program: &Program, source_code: String, file_name: String) -> AnalysisResult {
        let mut lint_visitor = LintVisitor::new(source_code, file_name.clone());
        program.visit_with(&mut lint_visitor);

        let mut decorator_visitor = DecoratorVisitor::new();
        program.visit_with(&mut decorator_visitor);

        let mut unsupported_visitor = UnsupportedApiVisitor::new(file_name);
        program.visit_with(&mut unsupported_visitor);

        AnalysisResult {
            errors: lint_visitor.errors,
            diagnostics: unsupported_visitor.diagnostics,
            graph: decorator_visitor.graph,
        }
    }
}
