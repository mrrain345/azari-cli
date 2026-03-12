use std::path::Path;

use serde::Deserialize;

use crate::receipt::{error::ReceiptError, field::ReceiptField, source::SourcePathGuard};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Receipt {
    pub from: ReceiptField,
    pub name: ReceiptField,
    pub hostname: ReceiptField,
}

impl Receipt {
    /// Parses a receipt from a YAML file.
    pub fn from_file(path: &Path) -> Result<Self, ReceiptError> {
        let file = std::fs::File::open(path)?;
        let _guard = SourcePathGuard::set_path(path.to_path_buf());
        Ok(serde_saphyr::from_reader(file)?)
    }
}
