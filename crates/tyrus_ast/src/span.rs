/// Source location for error reporting — traces back to the original TypeScript
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct TyrusSpan {
    pub start: u32,
    pub end: u32,
}

impl TyrusSpan {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn dummy() -> Self {
        Self { start: 0, end: 0 }
    }

    pub fn from_swc(span: swc_common::Span) -> Self {
        Self {
            start: span.lo.0,
            end: span.hi.0,
        }
    }
}
