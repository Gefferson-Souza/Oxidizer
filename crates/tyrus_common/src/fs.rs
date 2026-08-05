//! Filesystem newtypes (C-NEWTYPE): a path with transpiler-input meaning,
//! not an arbitrary `String`/`PathBuf`.

use std::path::{Path, PathBuf};

/// A path to a transpiler input file.
///
/// ```
/// use tyrus_common::fs::FilePath;
/// use std::path::Path;
///
/// let p = FilePath::from("src/main.ts");
/// assert_eq!(p.as_ref(), Path::new("src/main.ts"));
/// ```
#[derive(Debug, Clone)]
pub struct FilePath(pub PathBuf);

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl From<PathBuf> for FilePath {
    fn from(path: PathBuf) -> Self {
        FilePath(path)
    }
}

impl From<&str> for FilePath {
    fn from(path: &str) -> Self {
        FilePath(PathBuf::from(path))
    }
}
