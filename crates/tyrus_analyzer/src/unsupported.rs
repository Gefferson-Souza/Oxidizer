use swc_ecma_ast::{CallExpr, Callee, Expr, MemberExpr};
use swc_ecma_visit::{Visit, VisitWith};

use crate::severity::Diagnostic;

/// Detects usage of APIs that cannot be transpiled to Rust
pub(crate) struct UnsupportedApiVisitor {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) file_name: String,
}

const BLOCKED_GLOBALS: &[(&str, &str, &str)] = &[
    (
        "document",
        "tyrus::unsupported::dom",
        "DOM APIs are not available in Rust",
    ),
    (
        "window",
        "tyrus::unsupported::dom",
        "Browser window is not available in Rust",
    ),
    (
        "navigator",
        "tyrus::unsupported::dom",
        "Navigator API is not available in Rust",
    ),
    (
        "localStorage",
        "tyrus::unsupported::dom",
        "LocalStorage is not available in Rust",
    ),
    (
        "sessionStorage",
        "tyrus::unsupported::dom",
        "SessionStorage is not available in Rust",
    ),
    (
        "XMLHttpRequest",
        "tyrus::unsupported::dom",
        "XMLHttpRequest is not available in Rust — use reqwest",
    ),
];

const BLOCKED_FUNCTIONS: &[(&str, &str, &str, &str)] = &[
    (
        "require",
        "tyrus::unsupported::commonjs",
        "CommonJS require() is not supported",
        "Use ES module imports: import { x } from 'module'",
    ),
    (
        "setTimeout",
        "tyrus::unsupported::timer",
        "setTimeout is not directly supported",
        "Use tokio::time::sleep() for async delays",
    ),
    (
        "setInterval",
        "tyrus::unsupported::timer",
        "setInterval is not directly supported",
        "Use tokio::time::interval() for recurring tasks",
    ),
    (
        "clearTimeout",
        "tyrus::unsupported::timer",
        "clearTimeout is not directly supported",
        "Use tokio task cancellation",
    ),
    (
        "clearInterval",
        "tyrus::unsupported::timer",
        "clearInterval is not directly supported",
        "Use tokio task cancellation",
    ),
];

impl UnsupportedApiVisitor {
    pub(crate) fn new(file_name: String) -> Self {
        Self {
            diagnostics: Vec::new(),
            file_name,
        }
    }

    fn span_to_range(&self, span: swc_common::Span) -> (usize, usize) {
        (span.lo.0 as usize, span.hi.0 as usize)
    }
}

impl Visit for UnsupportedApiVisitor {
    fn visit_member_expr(&mut self, n: &MemberExpr) {
        if let Expr::Ident(ident) = &*n.obj {
            let name = ident.sym.as_ref();
            for (global, code, msg) in BLOCKED_GLOBALS {
                if name == *global {
                    let (start, end) = self.span_to_range(n.span);
                    self.diagnostics
                        .push(Diagnostic::error(code, msg).with_span(start, end, &self.file_name));
                }
            }
        }
        n.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, n: &CallExpr) {
        if let Callee::Expr(expr) = &n.callee {
            if let Expr::Ident(ident) = &**expr {
                let name = ident.sym.as_ref();
                for (func, code, msg, suggestion) in BLOCKED_FUNCTIONS {
                    if name == *func {
                        let (start, end) = self.span_to_range(n.span);
                        self.diagnostics.push(
                            Diagnostic::error(code, msg)
                                .with_span(start, end, &self.file_name)
                                .with_suggestion(suggestion),
                        );
                    }
                }
            }
        }
        n.visit_children_with(self);
    }
}
