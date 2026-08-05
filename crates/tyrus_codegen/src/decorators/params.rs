//! `@Body`, `@Param`, `@Query` handlers — Axum extractor emitters.
//!
//! All three live in one file because the feedback memory rule
//! ("never create a separate handler function per decorator name") treats
//! per-name handler files as the anti-pattern; the only thing that matters
//! is that the dispatch is registry-based, not scattered across hot paths.
//! Future param decorators (`@Headers`, `@Req`, `@Session`, ...) can keep
//! piling up here, or split out only when this file approaches the 400-line
//! limit.
//!
//! Each handler emits the parameter binding token used in the generated
//! Axum handler signature:
//!
//! ```text
//! @Body() dto: CreateUserDto      → axum::Json(dto): axum::Json<CreateUserDto>
//! @Param('id') id: string          → axum::extract::Path(id): axum::extract::Path<String>
//! @Query('page') page: string      → axum::extract::Query(page): axum::extract::Query<String>
//! ```

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use swc_ecma_ast::Param;
use tyrus_decorator_kinds::DecoratorKind;

use super::ParamDecoratorHandler;

pub(crate) struct BodyHandler;

impl ParamDecoratorHandler for BodyHandler {
    fn kind(&self) -> DecoratorKind {
        DecoratorKind::Body
    }

    fn emit_extractor(
        &self,
        _param: &Param,
        param_name: &Ident,
        param_type: &TokenStream,
    ) -> TokenStream {
        quote! { axum::Json(#param_name): axum::Json<#param_type> }
    }
}

pub(crate) struct ParamHandler;

impl ParamDecoratorHandler for ParamHandler {
    fn kind(&self) -> DecoratorKind {
        DecoratorKind::Param
    }

    fn emit_extractor(
        &self,
        _param: &Param,
        param_name: &Ident,
        param_type: &TokenStream,
    ) -> TokenStream {
        quote! { axum::extract::Path(#param_name): axum::extract::Path<#param_type> }
    }
}

pub(crate) struct QueryHandler;

impl ParamDecoratorHandler for QueryHandler {
    fn kind(&self) -> DecoratorKind {
        DecoratorKind::Query
    }

    fn emit_extractor(
        &self,
        _param: &Param,
        param_name: &Ident,
        param_type: &TokenStream,
    ) -> TokenStream {
        quote! { axum::extract::Query(#param_name): axum::extract::Query<#param_type> }
    }
}

/// `@Headers()` — emits the entire `axum::http::HeaderMap` as the parameter
/// binding. The user's TypeScript type annotation is ignored because Axum
/// only exposes `HeaderMap` for full-headers extraction. Handlers can then
/// query individual headers via `headers.get("name")`.
///
/// A name-scoped form (`@Headers('authorization') auth: string`) would
/// require extending [`super::ParamDecoratorHandler`] with a body-prelude
/// hook so the generated handler can extract the specific value before
/// the user code runs. That extension is intentionally deferred — adding
/// it here is independent of validating the registry pattern.
pub(crate) struct HeadersHandler;

impl ParamDecoratorHandler for HeadersHandler {
    fn kind(&self) -> DecoratorKind {
        DecoratorKind::Headers
    }

    fn emit_extractor(
        &self,
        _param: &Param,
        param_name: &Ident,
        _param_type: &TokenStream,
    ) -> TokenStream {
        quote! { #param_name: axum::http::HeaderMap }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::format_ident;

    fn dummy_param() -> Param {
        Param {
            span: swc_common::DUMMY_SP,
            decorators: vec![],
            pat: swc_ecma_ast::Pat::Ident(swc_ecma_ast::BindingIdent {
                id: swc_ecma_ast::Ident::new(
                    "x".into(),
                    swc_common::DUMMY_SP,
                    swc_common::SyntaxContext::default(),
                ),
                type_ann: None,
            }),
        }
    }

    #[test]
    fn body_handler_emits_json_extractor() {
        let h = BodyHandler;
        let name = format_ident!("dto");
        let ty = quote! { CreateUserDto };
        let token = h.emit_extractor(&dummy_param(), &name, &ty).to_string();
        assert!(token.contains("axum :: Json"));
        assert!(token.contains("dto"));
        assert!(token.contains("CreateUserDto"));
        assert_eq!(h.kind(), DecoratorKind::Body);
    }

    #[test]
    fn param_handler_emits_path_extractor() {
        let h = ParamHandler;
        let name = format_ident!("id");
        let ty = quote! { String };
        let token = h.emit_extractor(&dummy_param(), &name, &ty).to_string();
        assert!(token.contains("axum :: extract :: Path"));
        assert!(token.contains("id"));
        assert!(token.contains("String"));
        assert_eq!(h.kind(), DecoratorKind::Param);
    }

    #[test]
    fn query_handler_emits_query_extractor() {
        let h = QueryHandler;
        let name = format_ident!("page");
        let ty = quote! { String };
        let token = h.emit_extractor(&dummy_param(), &name, &ty).to_string();
        assert!(token.contains("axum :: extract :: Query"));
        assert!(token.contains("page"));
        assert_eq!(h.kind(), DecoratorKind::Query);
    }

    #[test]
    fn headers_handler_emits_header_map_param() {
        let h = HeadersHandler;
        let name = format_ident!("headers");
        // The user's type annotation is ignored — HeaderMap is the only Axum form.
        let ty = quote! { ThisShouldBeIgnored };
        let token = h.emit_extractor(&dummy_param(), &name, &ty).to_string();
        assert!(token.contains("axum :: http :: HeaderMap"));
        assert!(token.contains("headers"));
        assert!(
            !token.contains("ThisShouldBeIgnored"),
            "user's type annotation must NOT appear in the emitted extractor"
        );
        assert_eq!(h.kind(), DecoratorKind::Headers);
    }
}
