use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::receipt::{
    ReceiptError, ReceiptField, ReceiptImport,
    fields::{DistroField, FilesField, FromField, HostnameField, NameField, PackagesField},
    path::SourcePathGuard,
};

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Receipt {
    /// Must remain the first field processed during a build — every other
    /// field that emits Containerfile instructions reads the resolved
    /// [`Distro`](crate::distro::Distro) value from the builder.
    pub distro: DistroField,

    pub import: ReceiptImport,
    pub from: FromField,
    pub name: NameField,
    pub hostname: HostnameField,
    pub packages: PackagesField,
    pub files: FilesField,
}

impl Receipt {
    pub fn from_file(path: &Path) -> Result<Self, ReceiptError> {
        let mut seen = HashSet::new();
        Self::from_file_inner(path, &mut seen)
    }

    fn merge(self, other: Self) -> Self {
        Self {
            distro: self.distro.merge(other.distro),
            import: self.import.merge(other.import),
            from: self.from.merge(other.from),
            name: self.name.merge(other.name),
            hostname: self.hostname.merge(other.hostname),
            packages: self.packages.merge(other.packages),
            files: self.files.merge(other.files),
        }
    }

    fn from_file_inner(path: &Path, seen: &mut HashSet<PathBuf>) -> Result<Self, ReceiptError> {
        let canonical = path.canonicalize()?;

        if !seen.insert(canonical.clone()) {
            return Ok(Self::default());
        }

        let mut current = Self::parse_single(&canonical)?;
        let mut base = Self::default();

        while let Some(next) = current.import.process_next_import() {
            let imported = Self::from_file_inner(&next, seen)?;
            base = base.merge(imported);
        }

        Ok(base.merge(current))
    }

    fn parse_single(path: &Path) -> Result<Self, ReceiptError> {
        let file = std::fs::File::open(path)?;
        let _guard = SourcePathGuard::push_path(path.into());
        Ok(serde_saphyr::from_reader(file)?)
    }
}
