#![forbid(unsafe_code)]
#![allow(unused_assignments)]
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum TyrusError {
    #[error("IO Error: {0}")]
    #[diagnostic(code(tyrus::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Parsing Error: {message}")]
    #[diagnostic(code(tyrus::parse_error))]
    ParserError {
        message: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("{message}")]
        span: SourceSpan,
    },

    #[error("Lint Error: Rust does not support 'var'. Use 'let' or 'const'.")]
    #[diagnostic(code(tyrus::lint::no_var))]
    UseOfVar {
        #[source_code]
        src: NamedSource<String>,
        #[label("replace 'var' with 'let' or 'const'")]
        span: SourceSpan,
    },

    #[error("Lint Error: Rust requires strict typing. 'any' is not allowed.")]
    #[diagnostic(code(tyrus::lint::no_any))]
    UseOfAny {
        #[source_code]
        src: NamedSource<String>,
        #[label("specify a concrete type")]
        span: SourceSpan,
    },

    #[error("Lint Error: Code injection via 'eval' is unsafe and not supported in Rust.")]
    #[diagnostic(code(tyrus::lint::no_eval))]
    UseOfEval {
        #[source_code]
        src: NamedSource<String>,
        #[label("remove 'eval' usage")]
        span: SourceSpan,
    },

    #[error("Unsupported Feature: {feature} is not yet supported in Tyrus.")]
    #[diagnostic(code(tyrus::unsupported))]
    UnsupportedFeature {
        feature: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("this feature is pending implementation")]
        span: SourceSpan,
    },

    #[error(
        "Lint Error: Ambiguous main entrypoint — file declares `function main()` AND \
         contains top-level executable statements. Tyrus would either drop the \
         statements or duplicate the main function. Choose one: either move the \
         statements inside `main()`, or remove the user-declared `main` function."
    )]
    #[diagnostic(code(tyrus::lint::ambiguous_main_entrypoint))]
    AmbiguousMainEntrypoint {
        #[source_code]
        src: NamedSource<String>,
        #[label("first top-level statement here")]
        span: SourceSpan,
    },

    #[error("Formatting Error: {0}")]
    #[diagnostic(code(tyrus::fmt_error))]
    FormattingError(String),

    #[error("Validation Error: {0}")]
    #[diagnostic(code(tyrus::validation_error))]
    Validation(String),

    #[error("Unknown Error")]
    #[diagnostic(code(tyrus::unknown))]
    Unknown,
}
