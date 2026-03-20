use std::path::PathBuf;

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::unique::ReceiptUnique;

/// Field for the `image` key.
///
/// Specifies the image name (e.g. `docker.io/example/myimage`) used as a base
/// for `podman build -t` tags.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct ImageField(pub(crate) ReceiptUnique<String>);

impl ReceiptField for ImageField {
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

impl Build for ImageField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        if let Some(image) = self.value()? {
            builder.set_image(image);
        }
        Ok(())
    }
}
