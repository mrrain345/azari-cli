use std::collections::HashSet;
use std::path::{Path, PathBuf};

use merge::Merge;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::ReceiptField;
use crate::receipt::{
    ReceiptError, ReceiptImport,
    fields::{
        DistroField, FilesField, FromField, HostnameField, ImageField, InstallField, NameField,
        PackagesField, UsersField,
    },
    path::SourcePathGuard,
};

#[derive(Debug, Default, Deserialize, Merge)]
#[serde(default, rename_all = "kebab-case")]
pub struct Receipt {
    pub import: ReceiptImport,

    pub distro: DistroField,
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

impl Build for Receipt {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        // `distro` must be built first — it populates `builder.distro`,
        // which other fields read from during their build step.
        self.distro.build(builder)?;
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
}

impl Receipt {
    fn error(&self) -> Option<ReceiptError> {
        let errors: Vec<ReceiptError> = vec![
            self.distro.error(),
            self.image.error(),
            self.from.error(),
            self.name.error(),
            self.hostname.error(),
            self.users.error(),
            self.files.error(),
            self.preinstall.error(),
            self.packages.error(),
            self.postinstall.error(),
        ]
        .into_iter()
        .flatten()
        .collect();

        match errors.len() {
            0 => None,
            1 => errors.into_iter().next(),
            _ => Some(ReceiptError::Aggregate(errors)),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, ReceiptError> {
        let mut seen = HashSet::new();
        let receipt = Self::from_file_inner(path, &mut seen)?;

        if let Some(error) = receipt.error() {
            Err(error)
        } else {
            Ok(receipt)
        }
    }

    fn from_file_inner(path: &Path, seen: &mut HashSet<PathBuf>) -> Result<Self, ReceiptError> {
        let canonical = path.canonicalize()?;

        if !seen.insert(canonical.clone()) {
            return Ok(Self::default());
        }

        let mut current = Self::parse_single(&canonical)?;
        let mut base = Self::default();

        for next in std::mem::take(&mut current.import) {
            let imported = Self::from_file_inner(&next, seen)?;
            base.merge(imported);
        }

        base.merge(current);
        Ok(base)
    }

    fn parse_single(path: &Path) -> Result<Self, ReceiptError> {
        let file = std::fs::File::open(path)?;
        let _guard = SourcePathGuard::push_path(path.into());
        Ok(serde_saphyr::from_reader(file)?)
    }
}
