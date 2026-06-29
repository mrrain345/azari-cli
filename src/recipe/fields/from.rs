use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::unique::RecipeUnique;

/// # From
/// Override the default base image for the selected distro.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[schemars(example = "from: quay.io/fedora/fedora:41")]
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
