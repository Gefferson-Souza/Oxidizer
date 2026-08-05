//! Shared foundation types for the Tyrus workspace: newtypes ([`fs::FilePath`]),
//! configuration ([`config::Config`]) and case-conversion utilities
//! ([`util::to_snake_case`], [`util::to_pascal_case`]) consumed by the
//! analyzer, codegen and orchestrator crates.
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::missing_errors_doc)]

pub mod config;
pub mod fs;
pub mod util;
