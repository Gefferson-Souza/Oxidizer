//! Transpilation run configuration.

use serde::{Deserialize, Serialize};

/// Input/output locations for a transpilation run.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Path of the TypeScript entry file or source directory.
    pub input_path: String,
    /// Path where the generated Rust project is written.
    pub output_path: String,
}
