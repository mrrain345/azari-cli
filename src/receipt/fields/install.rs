use std::path::PathBuf;

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::list::ReceiptList;

/// Field for `preinstall` and `postinstall` keys.
///
/// Each entry is a shell command that is emitted as its own `RUN` instruction.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct InstallField(pub(crate) ReceiptList<String>);

impl ReceiptField for InstallField {
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

impl Build for InstallField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        for command in self.value()? {
            builder.push(format!("RUN {command}"));
        }
        Ok(())
    }
}
