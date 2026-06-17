use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::fields::ImportField;
use crate::receipt::{
    ReceiptError, ReceiptField,
    fields::{
        DistroField, FilesField, FromField, HostnameField, ImageField, InstallField, NameField,
        PackagesField, UsersField,
    },
    path::SourcePathGuard,
};

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Receipt {
    /// Must remain the first field processed during a build — every other
    /// field that emits Containerfile instructions reads the resolved
    /// [`Distro`](crate::distro::Distro) value from the builder.
    pub distro: DistroField,

    pub import: ImportField,
    pub image: ImageField,
    pub from: FromField,
    pub name: NameField,
    pub hostname: HostnameField,
    pub users: UsersField,
    pub files: FilesField,
    pub preinstall: InstallField,
    pub packages: PackagesField,
    pub postinstall: InstallField,
}

impl Receipt {
    pub fn from_file(path: &Path) -> Result<Self, ReceiptError> {
        let mut seen = HashSet::new();
        Self::from_file_inner(path, &mut seen)
    }

    pub fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        // `distro` must be built first — it populates `builder.distro`,
        // which other fields read from during their build step.
        self.distro.build(builder)?;

        self.import.build(builder)?;
        self.image.build(builder)?;
        self.from.build(builder)?;
        self.name.build(builder)?;
        self.hostname.build(builder)?;
        self.users.build(builder)?;
        self.files.build(builder)?;
        self.preinstall.build(builder)?;
        self.packages.build(builder)?;
        self.postinstall.build(builder)?;

        Ok(())
    }

    fn merge(self, other: Self) -> Self {
        Self {
            distro: self.distro.merge(other.distro),
            import: self.import.merge(other.import),
            image: self.image.merge(other.image),
            from: self.from.merge(other.from),
            name: self.name.merge(other.name),
            hostname: self.hostname.merge(other.hostname),
            users: self.users.merge(other.users),
            files: self.files.merge(other.files),
            preinstall: self.preinstall.merge(other.preinstall),
            packages: self.packages.merge(other.packages),
            postinstall: self.postinstall.merge(other.postinstall),
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
