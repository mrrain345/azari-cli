use std::path::PathBuf;

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::list::ReceiptList;

/// Field for the `packages` key.
///
/// Merges package lists from all imported receipts and emits a single
/// distro-specific `RUN` install instruction.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct PackagesField(pub(crate) ReceiptList<String>);



impl ReceiptField for PackagesField {
    type Value = Vec<String>;

    fn value(self) -> Result<Self::Value, ReceiptError> {
        self.0.value()
    }

    fn sources(&self) -> &[PathBuf] {
        self.0.sources()
    }

    fn merge(self, other: Self) -> Self {
        Self(self.0.merge(other.0))
    }
}

impl Build for PackagesField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        let distro = builder.distro()?;
        let packages = self.value()?;
        if !packages.is_empty() {
            let refs: Vec<&str> = packages.iter().map(String::as_str).collect();
            if let Some(instruction) = distro.install_packages(&refs) {
                builder.push(instruction);
            }
        }
        Ok(())
    }
}
