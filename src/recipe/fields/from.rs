use merge::Merge;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::unique::RecipeUnique;

/// Field for the `from` key.
///
/// Overrides the base OCI image. When absent the distro's default image is
/// used instead. Emits the `FROM` instruction.
#[derive(Debug, Default, Deserialize, Merge)]
#[serde(transparent)]
pub struct FromField(RecipeUnique<String>);

impl RecipeField for FromField {
    type Value = Option<String>;

    fn name() -> Option<&'static str> {
        Some("from")
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "from".to_string())
    }
}

impl Build for FromField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        let distro = builder.distro()?;
        let image = self
            .value()?
            .unwrap_or_else(|| distro.default_image().to_owned());
        builder.set_base_image(image.clone());
        builder.push(format!("FROM {image} as builder"));
        Ok(())
    }
}
