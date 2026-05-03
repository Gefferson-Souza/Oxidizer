//! `@HttpCode(201)` handler — sets the response status code on a method.

use swc_ecma_ast::{CallExpr, ClassMethod, Expr, Lit};
use tyrus_decorator_kinds::DecoratorKind;

use super::{MethodDecoratorContext, MethodDecoratorHandler};

pub(crate) struct HttpCodeHandler;

impl HttpCodeHandler {
    /// Validates that `value` is a finite, non-negative integer fitting in `u16`.
    /// Returns `Some(value as u16)` if accepted; `None` otherwise. Replaces the
    /// previous `value as u16` lossy cast — `@HttpCode(70000)` no longer wraps
    /// silently to a different code, and `@HttpCode(404.5)` is rejected.
    fn parse_status_code(value: f64) -> Option<u16> {
        if value.is_finite() && value >= 0.0 && value <= u16::MAX as f64 && value.fract() == 0.0 {
            Some(value as u16)
        } else {
            None
        }
    }
}

impl MethodDecoratorHandler for HttpCodeHandler {
    fn kind(&self) -> DecoratorKind {
        DecoratorKind::HttpCode
    }

    fn apply(&self, _method: &ClassMethod, call: &CallExpr, ctx: &mut MethodDecoratorContext) {
        let Some(arg) = call.args.first() else {
            return;
        };
        let Expr::Lit(Lit::Num(num)) = &*arg.expr else {
            return;
        };
        // Out-of-range / fractional / negative values are silently dropped.
        // The handler intentionally has no token-emission API — diagnosing the
        // invalid input falls to the caller (whose generated code will then
        // not carry a `@HttpCode` decoration, equivalent to omitting it).
        if let Some(code) = Self::parse_status_code(num.value) {
            ctx.http_code = Some(code);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_kind_is_http_code() {
        let h = HttpCodeHandler;
        assert_eq!(h.kind(), DecoratorKind::HttpCode);
        assert!(!DecoratorKind::HttpCode.is_http_method());
        assert!(DecoratorKind::HttpCode.axum_routing_fn_name().is_none());
    }

    #[test]
    fn parse_status_code_accepts_valid_integers() {
        assert_eq!(HttpCodeHandler::parse_status_code(0.0), Some(0));
        assert_eq!(HttpCodeHandler::parse_status_code(200.0), Some(200));
        assert_eq!(HttpCodeHandler::parse_status_code(404.0), Some(404));
        assert_eq!(HttpCodeHandler::parse_status_code(65535.0), Some(u16::MAX));
    }

    #[test]
    fn parse_status_code_rejects_invalid_values() {
        // Negative values would silently wrap under `as u16`.
        assert_eq!(HttpCodeHandler::parse_status_code(-1.0), None);
        // Out-of-range values would silently wrap.
        assert_eq!(HttpCodeHandler::parse_status_code(70000.0), None);
        assert_eq!(HttpCodeHandler::parse_status_code(99999.0), None);
        // Fractional values would silently truncate.
        assert_eq!(HttpCodeHandler::parse_status_code(404.5), None);
        assert_eq!(HttpCodeHandler::parse_status_code(404.999), None);
        // NaN / infinities are rejected.
        assert_eq!(HttpCodeHandler::parse_status_code(f64::NAN), None);
        assert_eq!(HttpCodeHandler::parse_status_code(f64::INFINITY), None);
        assert_eq!(HttpCodeHandler::parse_status_code(f64::NEG_INFINITY), None);
    }
}
