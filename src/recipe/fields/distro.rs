use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::BuildError;
use crate::builder::{Build, Builder};
use crate::distro::Distro;
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::unique::RecipeUnique;

/// # Distro
/// Target Linux distribution for the image.
///
/// This selects distro-specific defaults such as package manager behavior.
///
/// Possible values: `arch`, `debian`, `fedora`, `ubuntu`.
#[derive(Debug, Clone, Default, Deserialize, Merge, JsonSchema)]
#[serde(transparent)]
pub struct DistroField(RecipeUnique<String>);

impl RecipeField for DistroField {
    type Value = Option<String>;

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "distro".to_string())
    }
}

impl DistroField {
    /// Returns the distro as a `Distro` enum.
    pub fn distro(&self) -> Result<Distro, BuildError> {
        self.clone()
            .value()?
            .ok_or(BuildError::DistroNotSpecified)?
            .parse()
    }
}

impl Build for DistroField {
    fn build(self, builder: &mut Builder) -> Result<(), BuildError> {
        if let Some(distro) = self.value().ok().flatten() {
            let distro = distro.parse::<Distro>()?;
            builder.set_distro(distro)?
        }

        Ok(())
    }
}
