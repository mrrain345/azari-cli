use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::unique::RecipeUnique;

/// Field for the `distro` key.
///
/// Holds the name of the target Linux distribution. During the build this is
/// the **first** field processed: it resolves to a [`Distro`](crate::distro::Distro)
/// value stored in the builder that every subsequent field reads.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[serde(transparent)]
pub struct DistroField(RecipeUnique<String>);

impl RecipeField for DistroField {
    type Value = Option<String>;

    fn name() -> Option<&'static str> {
        Some("distro")
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "distro".to_string())
    }
}

impl Build for DistroField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        let distro_str = self.value()?.ok_or(RecipeError::DistroNotSpecified)?;
        builder.set_distro(distro_str.parse()?);
        Ok(())
    }
}
