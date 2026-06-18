use merge::Merge;
use serde::{Deserialize, Serialize};

use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;

/// A list field in a receipt file.
///
/// Items from every source are merged into a single flat list in source order.
/// Multiple sources defining this field is not a conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptList<T = String>(Vec<T>);

impl<T> ReceiptList<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self(values)
    }
}

impl<T> ReceiptField for ReceiptList<T> {
    type Value = Vec<T>;

    fn name() -> Option<&'static str> {
        None
    }

    fn value(self) -> Result<Self::Value, ReceiptError> {
        Ok(self.0)
    }

    fn error(&self) -> Option<ReceiptError> {
        None
    }
}

impl<T> Merge for ReceiptList<T> {
    fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl<T> Default for ReceiptList<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
