use merge::Merge;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::{ReceiptField, rename_field_error};
use crate::receipt::list::ReceiptList;

/// Field for `preinstall` and `postinstall` keys.
///
/// Each entry is a shell command that is emitted as its own `RUN` instruction.
#[derive(Debug, Default, Deserialize, Merge)]
#[serde(transparent)]
pub struct InstallField(ReceiptList<String>);

impl ReceiptField for InstallField {
    type Value = Vec<String>;

    fn name() -> Option<&'static str> {
        None
    }

    fn value(self) -> Result<Self::Value, ReceiptError> {
        self.0.value()
    }

    fn error(&self) -> Option<ReceiptError> {
        rename_field_error(self.0.error(), |_| "install".to_string())
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
