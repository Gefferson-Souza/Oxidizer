pub mod decl;
pub mod expr;
pub mod ident;
pub mod lower;
pub mod lower_type;
pub mod module;
pub mod span;
pub mod stmt;
pub mod types;

// Re-export main types at crate root
pub use decl::{
    Accessibility, ClassDecl, ClassMethod, ClassProperty, Constructor, ConstructorParam, Decorator,
    EnumDecl, EnumVariant, FunctionDecl, InterfaceDecl, InterfaceField, TypeAliasDecl, TyrusDecl,
};
pub use expr::{AssignOp, BinaryOp, Param, TyrusExpr, UnaryOp, UpdateOp};
pub use ident::Ident;
pub use lower::lower_program;
pub use module::TyrusModule;
pub use span::TyrusSpan;
pub use stmt::TyrusStmt;
pub use types::TyrusType;
