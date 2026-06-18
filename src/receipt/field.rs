use merge::Merge;

use crate::receipt::error::ReceiptError;

/// Trait for all field types in a receipt file.
pub trait ReceiptField: Sized + Merge {
    /// Type of value this field resolves to.
    type Value;

    /// Returns the name of this field, if it has one.
    fn name() -> Option<&'static str>;

    /// Resolves this field into its value.
    fn value(self) -> Result<Self::Value, ReceiptError>;

    /// Returns any errors associated with this field.
    fn error(&self) -> Option<ReceiptError>;
}

pub(crate) fn rename_field_error<F>(error: Option<ReceiptError>, renamer: F) -> Option<ReceiptError>
where
    F: Fn(Option<String>) -> String,
{
    fn rename_inner(
        error: Option<ReceiptError>,
        renamer: &dyn Fn(Option<String>) -> String,
    ) -> Option<ReceiptError> {
        error.map(|e| match e {
            ReceiptError::FieldConflict { field, paths } => ReceiptError::FieldConflict {
                field: Some(renamer(field)),
                paths,
            },
            ReceiptError::Aggregate(errors) => ReceiptError::Aggregate(
                errors
                    .into_iter()
                    .map(|e| rename_inner(Some(e), renamer).unwrap())
                    .collect(),
            ),
            other => other,
        })
    }

    rename_inner(error, &renamer)
}
