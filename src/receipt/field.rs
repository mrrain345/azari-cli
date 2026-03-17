use std::path::PathBuf;

use crate::receipt::error::ReceiptError;

/// Trait for all field types in a receipt file.
pub trait ReceiptField: Sized {
    /// Type of value this field resolves to.
    type Value;

    /// Resolves this field into its value.
    fn value(self) -> Result<Self::Value, ReceiptError>;

    /// Returns the paths of all source files that defined this field.
    fn sources(&self) -> &[PathBuf];

    /// Merges another instance into this one, combining their sources.
    fn merge(self, other: Self) -> Self;
}
