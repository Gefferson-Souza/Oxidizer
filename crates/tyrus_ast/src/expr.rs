use crate::ident::Ident;
use crate::span::TyrusSpan;
use crate::types::TyrusType;

/// All expressions in the Oxidizable Standard
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TyrusExpr {
    // -- Literals --
    NumberLit(f64),
    StringLit(String),
    BoolLit(bool),
    ArrayLit(Vec<TyrusExpr>),
    ObjectLit(Vec<(Ident, TyrusExpr)>),
    TemplateLit(Vec<TemplatePart>),
    NullLit,

    // -- References --
    Ident(Ident),

    // -- Operations --
    Binary {
        left: Box<TyrusExpr>,
        op: BinaryOp,
        right: Box<TyrusExpr>,
        span: TyrusSpan,
    },
    Unary {
        op: UnaryOp,
        arg: Box<TyrusExpr>,
        span: TyrusSpan,
    },
    Paren(Box<TyrusExpr>),

    // -- Access --
    Member {
        object: Box<TyrusExpr>,
        property: Ident,
        optional: bool,
        span: TyrusSpan,
    },
    Index {
        object: Box<TyrusExpr>,
        index: Box<TyrusExpr>,
        span: TyrusSpan,
    },

    // -- Calls --
    Call {
        callee: Box<TyrusExpr>,
        args: Vec<TyrusExpr>,
        type_args: Vec<TyrusType>,
        span: TyrusSpan,
    },
    MethodCall {
        object: Box<TyrusExpr>,
        method: Ident,
        args: Vec<TyrusExpr>,
        span: TyrusSpan,
    },

    // -- Functions --
    Arrow {
        params: Vec<Param>,
        body: ArrowBody,
        is_async: bool,
        span: TyrusSpan,
    },

    // -- Control --
    Ternary {
        test: Box<TyrusExpr>,
        consequent: Box<TyrusExpr>,
        alternate: Box<TyrusExpr>,
        span: TyrusSpan,
    },

    // -- Assignment --
    Assign {
        target: Box<TyrusExpr>,
        value: Box<TyrusExpr>,
        op: AssignOp,
        span: TyrusSpan,
    },

    // -- Update --
    Update {
        arg: Box<TyrusExpr>,
        op: UpdateOp,
        prefix: bool,
        span: TyrusSpan,
    },

    // -- Await --
    Await(Box<TyrusExpr>),

    // -- Spread --
    Spread(Box<TyrusExpr>),

    // -- Type assertion (as) --
    TypeAssertion {
        expr: Box<TyrusExpr>,
        target_type: TyrusType,
        span: TyrusSpan,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TemplatePart {
    Str(String),
    Expr(TyrusExpr),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Param {
    pub name: Ident,
    pub ty: TyrusType,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ArrowBody {
    Expr(Box<TyrusExpr>),
    Block(Vec<crate::stmt::TyrusStmt>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    Eq,
    NotEq,
    StrictEq,
    StrictNotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    TypeOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum UpdateOp {
    Increment,
    Decrement,
}
