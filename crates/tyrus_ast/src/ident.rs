use crate::span::TyrusSpan;

/// A named identifier with source location
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Ident {
    pub name: String,
    pub span: TyrusSpan,
}

impl Ident {
    pub fn new(name: &str, span: TyrusSpan) -> Self {
        Self {
            name: name.to_string(),
            span,
        }
    }

    pub fn synthetic(name: &str) -> Self {
        Self {
            name: name.to_string(),
            span: TyrusSpan::dummy(),
        }
    }

    pub fn from_swc(ident: &swc_ecma_ast::Ident) -> Self {
        Self {
            name: ident.sym.to_string(),
            span: TyrusSpan::from_swc(ident.span),
        }
    }
}
