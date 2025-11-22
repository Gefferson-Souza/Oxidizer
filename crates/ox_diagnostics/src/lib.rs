use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum OxidizerError {
    #[error("IO Error: {0}")]
    #[diagnostic(code(oxidizer::io_error))]
    IoError(#[from] std::io::Error),

    #[error("Parsing Error: {message}")]
    #[diagnostic(code(oxidizer::parse_error))]
    ParserError {
        message: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("{message}")]
        span: SourceSpan,
    },
    
    #[error("Unknown Error")]
    #[diagnostic(code(oxidizer::unknown))]
    Unknown,
}
