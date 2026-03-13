use crate::expr::TyrusExpr;
use crate::ident::Ident;
use crate::span::TyrusSpan;
use crate::types::TyrusType;

/// All statements in the Oxidizable Standard
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TyrusStmt {
    /// `let x: T = expr;` or `const x: T = expr;`
    VarDecl {
        name: Ident,
        ty: TyrusType,
        init: Option<TyrusExpr>,
        mutable: bool,
        span: TyrusSpan,
    },

    /// Expression statement: `foo();`
    Expr(TyrusExpr),

    /// `return expr;`
    Return {
        value: Option<TyrusExpr>,
        span: TyrusSpan,
    },

    /// `if (test) { body } else { alt }`
    If {
        test: TyrusExpr,
        body: Vec<TyrusStmt>,
        alt: Option<Vec<TyrusStmt>>,
        span: TyrusSpan,
    },

    /// `while (test) { body }`
    While {
        test: TyrusExpr,
        body: Vec<TyrusStmt>,
        span: TyrusSpan,
    },

    /// `for (init; test; update) { body }`
    For {
        init: Option<Box<TyrusStmt>>,
        test: Option<TyrusExpr>,
        update: Option<TyrusExpr>,
        body: Vec<TyrusStmt>,
        span: TyrusSpan,
    },

    /// `for (const x of iter) { body }`
    ForOf {
        binding: Ident,
        iter: TyrusExpr,
        body: Vec<TyrusStmt>,
        span: TyrusSpan,
    },

    /// `do { body } while (test);`
    DoWhile {
        body: Vec<TyrusStmt>,
        test: TyrusExpr,
        span: TyrusSpan,
    },

    /// `switch (discriminant) { cases }`
    Switch {
        discriminant: TyrusExpr,
        cases: Vec<SwitchCase>,
        span: TyrusSpan,
    },

    /// `break;`
    Break(TyrusSpan),

    /// `continue;`
    Continue(TyrusSpan),

    /// Block of statements `{ ... }`
    Block(Vec<TyrusStmt>),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SwitchCase {
    pub test: Option<TyrusExpr>,
    pub body: Vec<TyrusStmt>,
}
