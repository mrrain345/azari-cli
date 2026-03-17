use std::path::PathBuf;

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::unique::ReceiptUnique;

/// Field for the `from` key.
///
/// Overrides the base OCI image. When absent the distro's default image is
/// used instead. Emits the `FROM` instruction.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct FromField(pub(crate) ReceiptUnique<String>);

impl ReceiptField for FromField {
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

impl Build for FromField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        let distro = builder.distro()?;
        let image = self
            .value()?
            .unwrap_or_else(|| distro.default_image().to_owned());
        builder.push(format!("FROM {image}"));
        Ok(())
    }
}
