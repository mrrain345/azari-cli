use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::unique::RecipeUnique;

/// # Image
/// Name of the output image.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[schemars(example = "image: ghcr.io/username/image")]
#[serde(transparent)]
pub struct ImageField(RecipeUnique<String>);

impl RecipeField for ImageField {
    type Value = Option<String>;

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "image".to_string())
    }
}

impl Build for ImageField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        if let Some(image) = self.value()? {
            builder.set_image(image);
        }
        Ok(())
    }
}
