use merge::Merge;

use crate::receipt::error::ReceiptError;

/// Trait for all field types in a receipt file.
pub trait ReceiptField: Sized + Merge {
    /// Type of value this field resolves to.
    type Value;

    /// Resolves this field into its value.
    fn value(self) -> Result<Self::Value, ReceiptError>;
}
