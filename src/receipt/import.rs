use std::path::PathBuf;

use merge::Merge;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::receipt::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::path::current_path;

/// Import field state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReceiptImport {
    imports: Vec<PathBuf>,
}

impl ReceiptImport {
    /// Creates import state from one source receipt and raw `import` entries.
    /// Relative paths are resolved against the source receipt directory.
    pub fn new(source: PathBuf, imports: Vec<PathBuf>) -> Result<Self, ReceiptError> {
        let source = source.canonicalize().unwrap_or(source);

        let base_dir = source
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| ReceiptError::InvalidReceiptPath(source.clone()))?;

        let mut resolved = Vec::with_capacity(imports.len());
        for p in imports {
            let path = if p.is_absolute() { p } else { base_dir.join(p) };
            resolved.push(path.canonicalize()?);
        }

        Ok(Self { imports: resolved })
    }
}

impl ReceiptField for ReceiptImport {
    type Value = Vec<PathBuf>;

    fn name() -> Option<&'static str> {
        Some("import")
    }

    /// Imports (empty when full load completes).
    fn value(self) -> Result<Self::Value, ReceiptError> {
        Ok(self.imports)
    }

    fn error(&self) -> Option<ReceiptError> {
        None
    }
}

impl IntoIterator for ReceiptImport {
    type Item = PathBuf;
    type IntoIter = std::vec::IntoIter<PathBuf>;

    fn into_iter(self) -> Self::IntoIter {
        self.imports.into_iter()
    }
}

impl Merge for ReceiptImport {
    fn merge(&mut self, other: Self) {
        self.imports.extend(other.imports);
    }
}

impl<'de> Deserialize<'de> for ReceiptImport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let imports = Option::<Vec<PathBuf>>::deserialize(deserializer)?.unwrap_or_default();
        let source = current_path()
            .ok_or_else(|| serde::de::Error::custom("current source path is not set"))?;
        ReceiptImport::new(source, imports).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ReceiptImport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Option::<()>::None.serialize(serializer)
    }
}
