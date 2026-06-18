use merge::Merge;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::{ReceiptField, rename_field_error};
use crate::receipt::list::ReceiptList;

/// Field for the `packages` key.
///
/// Merges package lists from all imported receipts and emits a single
/// distro-specific `RUN` install instruction.
#[derive(Debug, Default, Deserialize, Merge)]
#[serde(transparent)]
pub struct PackagesField(ReceiptList<String>);

impl ReceiptField for PackagesField {
    type Value = Vec<String>;

    fn name() -> Option<&'static str> {
        Some("packages")
    }

    fn value(self) -> Result<Self::Value, ReceiptError> {
        self.0.value()
    }

    fn error(&self) -> Option<ReceiptError> {
        rename_field_error(self.0.error(), |_| "packages".to_string())
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
