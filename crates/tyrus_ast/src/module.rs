use crate::decl::TyrusDecl;
use crate::stmt::TyrusStmt;

/// Top-level container for a transpiled file
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TyrusModule {
    pub declarations: Vec<TyrusDecl>,
    pub statements: Vec<TyrusStmt>,
    pub imports: Vec<ImportDecl>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ImportDecl {
    pub source: String,
    pub specifiers: Vec<ImportSpecifier>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ImportSpecifier {
    Named { local: String, imported: String },
    Default(String),
    Namespace(String),
}

impl TyrusModule {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            statements: Vec::new(),
            imports: Vec::new(),
        }
    }
}

impl Default for TyrusModule {
    fn default() -> Self {
        Self::new()
    }
}
