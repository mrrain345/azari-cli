use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::ReceiptImport;
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;

/// Field for the `from` key.
///
/// Overrides the base OCI image. When absent the distro's default image is
/// used instead. Emits the `FROM` instruction.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct ImportField(ReceiptImport);

impl ReceiptField for ImportField {
    type Value = Vec<PathBuf>;

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

impl Deref for ImportField {
    type Target = ReceiptImport;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ImportField {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Build for ImportField {
    fn build(self, _builder: &mut Builder) -> Result<(), ReceiptError> {
        Ok(())
    }
}
