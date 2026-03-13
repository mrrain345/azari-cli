use std::path::PathBuf;

use crate::receipt::error::ReceiptError;

/// Trait for all field types in a receipt file.
pub trait ReceiptField {
    /// The resolved value type returned by [`value`](ReceiptField::value).
    type Value<'a>: 'a
    where
        Self: 'a;

    /// Returns the resolved value, or `Err` if it cannot be resolved.
    fn value(&self) -> Result<Self::Value<'_>, ReceiptError>;

    /// Returns the paths of all source files that defined this field.
    fn sources(&self) -> &[PathBuf];

    /// Merges another instance into this one, combining their sources.
    fn merge(self, other: Self) -> Self;
}
