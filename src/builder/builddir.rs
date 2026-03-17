use std::path::{Path, PathBuf};

use crate::receipt::ReceiptError;

enum BuildDirInner {
    /// Temporary directory, removed automatically on drop.
    Temp(tempfile::TempDir),
    /// User-specified directory, preserved after the build.
    Persistent(PathBuf),
}

impl std::fmt::Debug for BuildDirInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildDirInner::Temp(dir) => f.debug_tuple("Temp").field(&dir.path()).finish(),
            BuildDirInner::Persistent(path) => f.debug_tuple("Persistent").field(path).finish(),
        }
    }
}

/// Manages the build directory lifecycle.
///
/// When created without a path (via [`BuildDir::temp`]), a temporary directory
/// is used and cleaned up automatically on drop.
///
/// When created with an explicit path (via [`BuildDir::persistent`]), the
/// directory is preserved after the build — useful for inspecting intermediate
/// files during debugging.
#[derive(Debug)]
pub struct BuildDir {
    inner: BuildDirInner,
}

impl BuildDir {
    /// Creates a temporary build directory that is removed on drop.
    pub fn temp() -> Result<Self, ReceiptError> {
        let dir = tempfile::tempdir()?;
        Ok(BuildDir {
            inner: BuildDirInner::Temp(dir),
        })
    }

    /// Creates a persistent build directory at the given path.
    ///
    /// The path must point to an empty or non-existent directory.
    /// Returns [`ReceiptError::BuildDirNotEmpty`] if it already contains files.
    pub fn persistent(path: PathBuf) -> Result<Self, ReceiptError> {
        if path.exists() {
            let mut entries = std::fs::read_dir(&path)?;
            if entries.next().is_some() {
                return Err(ReceiptError::BuildDirNotEmpty(path));
            }
        } else {
            std::fs::create_dir_all(&path)?;
        }
        Ok(BuildDir {
            inner: BuildDirInner::Persistent(path),
        })
    }

    /// Returns the path to the build directory.
    pub fn path(&self) -> &Path {
        match &self.inner {
            BuildDirInner::Temp(dir) => dir.path(),
            BuildDirInner::Persistent(path) => path.as_path(),
        }
    }
}
