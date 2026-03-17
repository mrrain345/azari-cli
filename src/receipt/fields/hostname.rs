use std::path::PathBuf;

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::unique::ReceiptUnique;

/// Field for the `hostname` key.
///
/// Emits a distro-specific `RUN` instruction to set the container's hostname.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct HostnameField(pub(crate) ReceiptUnique<String>);



impl ReceiptField for HostnameField {
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

impl Build for HostnameField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        let distro = builder.distro()?;
        if let Some(instruction) = self.value()?.and_then(|h| distro.set_hostname(&h)) {
            builder.push(instruction);
        }
        Ok(())
    }
}
