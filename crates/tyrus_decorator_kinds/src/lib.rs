//! Single source of truth for NestJS decorator name → kind classification.
//!
//! This crate is intentionally lightweight (zero dependencies) so it can be
//! shared by both `tyrus_analyzer` and `tyrus_codegen` without dragging
//! `proc_macro2`/`quote` into the analyzer.
//!
//! Usage:
//! ```
//! use tyrus_decorator_kinds::{DecoratorKind, DecoratorScope};
//!
//! assert_eq!(DecoratorKind::from_name("Controller"), Some(DecoratorKind::Controller));
//! assert_eq!(DecoratorKind::Controller.scope(), DecoratorScope::Class);
//! assert_eq!(DecoratorKind::from_name("UnknownDecorator"), None);
//! ```

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo)]

/// Where in the AST a decorator can appear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecoratorScope {
    /// Class-level: `@Controller`, `@Injectable`, `@Module`, `@UseGuards`.
    Class,
    /// Method-level: `@Get`, `@Post`, `@HttpCode`, etc.
    Method,
    /// Parameter-level: `@Body`, `@Param`, `@Query`, `@Headers`.
    Param,
}

/// Every NestJS decorator the transpiler currently recognizes.
///
/// Adding a new decorator means adding ONE variant here plus mapping its name
/// in [`DecoratorKind::from_name`]. The handler that emits Rust for it lives
/// in `tyrus_codegen::decorators`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecoratorKind {
    // Class-level
    Module,
    Injectable,
    Controller,
    UseGuards,

    // Method-level
    HttpGet,
    HttpPost,
    HttpPut,
    HttpDelete,
    HttpPatch,
    HttpCode,

    // Param-level
    Body,
    Param,
    Query,
    Headers,
}

impl DecoratorKind {
    /// Maps a decorator identifier (the bare name as written in TypeScript) to
    /// its kind. Returns `None` for unknown decorators — callers are expected
    /// to fall through to a generic translation, never to error out.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Module" => Some(Self::Module),
            "Injectable" => Some(Self::Injectable),
            "Controller" => Some(Self::Controller),
            "UseGuards" => Some(Self::UseGuards),
            "Get" => Some(Self::HttpGet),
            "Post" => Some(Self::HttpPost),
            "Put" => Some(Self::HttpPut),
            "Delete" => Some(Self::HttpDelete),
            "Patch" => Some(Self::HttpPatch),
            "HttpCode" => Some(Self::HttpCode),
            "Body" => Some(Self::Body),
            "Param" => Some(Self::Param),
            "Query" => Some(Self::Query),
            "Headers" => Some(Self::Headers),
            _ => None,
        }
    }

    /// Where this decorator is allowed to appear in the AST.
    pub fn scope(self) -> DecoratorScope {
        match self {
            Self::Module | Self::Injectable | Self::Controller | Self::UseGuards => {
                DecoratorScope::Class
            }
            Self::HttpGet
            | Self::HttpPost
            | Self::HttpPut
            | Self::HttpDelete
            | Self::HttpPatch
            | Self::HttpCode => DecoratorScope::Method,
            Self::Body | Self::Param | Self::Query | Self::Headers => DecoratorScope::Param,
        }
    }

    /// True iff this decorator selects an HTTP verb (Get/Post/Put/Delete/Patch).
    /// Used by routing to dispatch to the right `axum::routing::*` constructor.
    pub fn is_http_method(self) -> bool {
        matches!(
            self,
            Self::HttpGet | Self::HttpPost | Self::HttpPut | Self::HttpDelete | Self::HttpPatch
        )
    }

    /// Returns the `axum::routing::*` constructor name for HTTP verb
    /// decorators (e.g. `HttpGet` → `"get"`). Returns `None` for any
    /// non-verb kind.
    ///
    /// This lives in `tyrus_decorator_kinds` (rather than the codegen crate)
    /// because the mapping is a property of the decorator, not of the code
    /// emission strategy — keeping it here means routing.rs and the registry
    /// share the same source of truth.
    pub fn axum_routing_fn_name(self) -> Option<&'static str> {
        match self {
            Self::HttpGet => Some("get"),
            Self::HttpPost => Some("post"),
            Self::HttpPut => Some("put"),
            Self::HttpDelete => Some("delete"),
            Self::HttpPatch => Some("patch"),
            _ => None,
        }
    }

    /// The original TypeScript name, for diagnostics.
    pub fn name(self) -> &'static str {
        match self {
            Self::Module => "Module",
            Self::Injectable => "Injectable",
            Self::Controller => "Controller",
            Self::UseGuards => "UseGuards",
            Self::HttpGet => "Get",
            Self::HttpPost => "Post",
            Self::HttpPut => "Put",
            Self::HttpDelete => "Delete",
            Self::HttpPatch => "Patch",
            Self::HttpCode => "HttpCode",
            Self::Body => "Body",
            Self::Param => "Param",
            Self::Query => "Query",
            Self::Headers => "Headers",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_name_known_decorators() {
        assert_eq!(
            DecoratorKind::from_name("Controller"),
            Some(DecoratorKind::Controller)
        );
        assert_eq!(
            DecoratorKind::from_name("Get"),
            Some(DecoratorKind::HttpGet)
        );
        assert_eq!(DecoratorKind::from_name("Body"), Some(DecoratorKind::Body));
        assert_eq!(
            DecoratorKind::from_name("UseGuards"),
            Some(DecoratorKind::UseGuards)
        );
    }

    #[test]
    fn from_name_unknown_returns_none() {
        assert_eq!(DecoratorKind::from_name("IsEmail"), None);
        assert_eq!(DecoratorKind::from_name(""), None);
        assert_eq!(DecoratorKind::from_name("controller"), None); // case-sensitive
    }

    #[test]
    fn scope_classification() {
        assert_eq!(DecoratorKind::Controller.scope(), DecoratorScope::Class);
        assert_eq!(DecoratorKind::HttpGet.scope(), DecoratorScope::Method);
        assert_eq!(DecoratorKind::Body.scope(), DecoratorScope::Param);
    }

    #[test]
    fn http_method_classification() {
        assert!(DecoratorKind::HttpGet.is_http_method());
        assert!(DecoratorKind::HttpPost.is_http_method());
        assert!(!DecoratorKind::HttpCode.is_http_method());
        assert!(!DecoratorKind::Controller.is_http_method());
    }

    #[test]
    fn axum_routing_fn_name_for_verbs() {
        assert_eq!(DecoratorKind::HttpGet.axum_routing_fn_name(), Some("get"));
        assert_eq!(DecoratorKind::HttpPost.axum_routing_fn_name(), Some("post"));
        assert_eq!(DecoratorKind::HttpPut.axum_routing_fn_name(), Some("put"));
        assert_eq!(
            DecoratorKind::HttpDelete.axum_routing_fn_name(),
            Some("delete")
        );
        assert_eq!(
            DecoratorKind::HttpPatch.axum_routing_fn_name(),
            Some("patch")
        );
        assert_eq!(DecoratorKind::HttpCode.axum_routing_fn_name(), None);
        assert_eq!(DecoratorKind::Controller.axum_routing_fn_name(), None);
    }

    #[test]
    fn name_roundtrip() {
        for kind in [
            DecoratorKind::Module,
            DecoratorKind::Injectable,
            DecoratorKind::Controller,
            DecoratorKind::UseGuards,
            DecoratorKind::HttpGet,
            DecoratorKind::HttpPost,
            DecoratorKind::HttpPut,
            DecoratorKind::HttpDelete,
            DecoratorKind::HttpPatch,
            DecoratorKind::HttpCode,
            DecoratorKind::Body,
            DecoratorKind::Param,
            DecoratorKind::Query,
            DecoratorKind::Headers,
        ] {
            assert_eq!(DecoratorKind::from_name(kind.name()), Some(kind));
        }
    }
}
