use std::path::PathBuf;

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::unique::ReceiptUnique;

/// Field for the `distro` key.
///
/// Holds the name of the target Linux distribution. During the build this is
/// the **first** field processed: it resolves to a [`Distro`](crate::distro::Distro)
/// value stored in the builder that every subsequent field reads.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct DistroField(pub(crate) ReceiptUnique<String>);

impl ReceiptField for DistroField {
    type Value = Option<String>;

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

impl Build for DistroField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        let distro_str = self.value()?.ok_or(ReceiptError::DistroNotSpecified)?;
        builder.set_distro(distro_str.parse()?);
        Ok(())
    }
}
