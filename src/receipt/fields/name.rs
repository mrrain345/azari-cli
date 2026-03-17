use std::path::PathBuf;

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::unique::ReceiptUnique;

/// Field for the `name` key.
///
/// Sets the `org.opencontainers.image.title` OCI label on the image.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct NameField(pub(crate) ReceiptUnique<String>);



impl ReceiptField for NameField {
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

impl Build for NameField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        if let Some(name) = self.value()? {
            builder.push(format!(
                "LABEL org.opencontainers.image.title=\"{}\"",
                escape(&name)
            ));
        }
        Ok(())
    }
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
