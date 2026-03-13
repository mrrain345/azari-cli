use std::path::Path;

use serde::Deserialize;

use crate::receipt::{ReceiptError, ReceiptList, ReceiptUnique, path::SourcePathGuard};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Receipt {
    pub from: ReceiptUnique,
    pub name: ReceiptUnique,
    pub hostname: ReceiptUnique,

    pub packages: ReceiptList,
}

impl Receipt {
    /// Parses a receipt from a YAML file.
    pub fn from_file(path: &Path) -> Result<Self, ReceiptError> {
        let file = std::fs::File::open(path)?;
        let _guard = SourcePathGuard::push_path(path.to_path_buf());
        Ok(serde_saphyr::from_reader(file)?)
    }
}
