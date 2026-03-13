use crate::expr::{Param, TyrusExpr};
use crate::ident::Ident;
use crate::span::TyrusSpan;
use crate::stmt::TyrusStmt;
use crate::types::TyrusType;

/// Top-level declarations
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TyrusDecl {
    Function(FunctionDecl),
    Class(ClassDecl),
    Interface(InterfaceDecl),
    Enum(EnumDecl),
    TypeAlias(TypeAliasDecl),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FunctionDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: TyrusType,
    pub body: Vec<TyrusStmt>,
    pub is_async: bool,
    pub is_exported: bool,
    pub type_params: Vec<Ident>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ClassDecl {
    pub name: Ident,
    pub properties: Vec<ClassProperty>,
    pub methods: Vec<ClassMethod>,
    pub constructor: Option<Constructor>,
    pub decorators: Vec<Decorator>,
    pub is_exported: bool,
    pub type_params: Vec<Ident>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ClassProperty {
    pub name: Ident,
    pub ty: TyrusType,
    pub accessibility: Accessibility,
    pub is_readonly: bool,
    pub init: Option<TyrusExpr>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ClassMethod {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: TyrusType,
    pub body: Vec<TyrusStmt>,
    pub is_async: bool,
    pub accessibility: Accessibility,
    pub decorators: Vec<Decorator>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Constructor {
    pub params: Vec<ConstructorParam>,
    pub body: Vec<TyrusStmt>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ConstructorParam {
    pub name: Ident,
    pub ty: TyrusType,
    pub accessibility: Option<Accessibility>,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Accessibility {
    Public,
    Private,
    Protected,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Decorator {
    pub name: String,
    pub args: Vec<TyrusExpr>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct InterfaceDecl {
    pub name: Ident,
    pub fields: Vec<InterfaceField>,
    pub is_exported: bool,
    pub type_params: Vec<Ident>,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct InterfaceField {
    pub name: Ident,
    pub ty: TyrusType,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EnumDecl {
    pub name: Ident,
    pub variants: Vec<EnumVariant>,
    pub is_exported: bool,
    pub is_string: bool,
    pub span: TyrusSpan,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EnumVariant {
    pub name: Ident,
    pub value: Option<TyrusExpr>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TypeAliasDecl {
    pub name: Ident,
    pub ty: TyrusType,
    pub is_exported: bool,
    pub span: TyrusSpan,
}
