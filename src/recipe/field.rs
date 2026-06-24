use merge::Merge;

use crate::recipe::error::RecipeError;

/// Trait for all field types in a recipe file.
pub trait RecipeField: Sized + Merge {
    /// Type of value this field resolves to.
    type Value;

    /// Returns the name of this field, if it has one.
    fn name() -> Option<&'static str>;

    /// Resolves this field into its value.
    fn value(self) -> Result<Self::Value, RecipeError>;

    /// Returns any errors associated with this field.
    fn error(&self) -> Option<RecipeError>;
}

pub(crate) fn rename_field_error<F>(error: Option<RecipeError>, renamer: F) -> Option<RecipeError>
where
    F: Fn(Option<String>) -> String,
{
    fn rename_inner(
        error: Option<RecipeError>,
        renamer: &dyn Fn(Option<String>) -> String,
    ) -> Option<RecipeError> {
        error.map(|e| match e {
            RecipeError::FieldConflict { field, paths } => RecipeError::FieldConflict {
                field: Some(renamer(field)),
                paths,
            },
            RecipeError::Aggregate(errors) => RecipeError::Aggregate(
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
