use std::path::PathBuf;

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
    pending: Vec<PathBuf>,
    loaded: Vec<PathBuf>,
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

        let mut pending = Vec::with_capacity(imports.len());
        for p in imports {
            let resolved = if p.is_absolute() { p } else { base_dir.join(p) };
            pending.push(resolved.canonicalize()?);
        }

        Ok(Self {
            pending,
            loaded: Vec::new(),
        })
    }

    /// Pops the next import to process and marks it as loaded.
    /// Already loaded imports are skipped.
    pub(crate) fn process_next_import(&mut self) -> Option<PathBuf> {
        while !self.pending.is_empty() {
            let path = self.pending.remove(0);
            if self.loaded.iter().any(|p| p == &path) {
                continue;
            }
            self.loaded.push(path.clone());
            return Some(path);
        }
        None
    }
}

impl ReceiptField for ReceiptImport {
    type Value = Vec<PathBuf>;

    /// Pending imports (empty when full load completes).
    fn value(self) -> Result<Self::Value, ReceiptError> {
        Ok(self.pending)
    }

    /// Loaded imported module paths.
    fn sources(&self) -> &[PathBuf] {
        &self.loaded
    }

    fn merge(self, other: Self) -> Self {
        let mut pending = self.pending;
        pending.extend(other.pending);

        let mut loaded = self.loaded;
        for p in other.loaded {
            if !loaded.iter().any(|x| x == &p) {
                loaded.push(p);
            }
        }

        Self { pending, loaded }
    }
}

impl<'de> Deserialize<'de> for ReceiptImport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let imports = Option::<Vec<PathBuf>>::deserialize(deserializer)?.unwrap_or_default();
        let source = current_path().expect("Current source path is not set");
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
