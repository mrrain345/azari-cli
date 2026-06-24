use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::unique::RecipeUnique;

/// Field for the `hostname` key.
///
/// Emits a distro-specific `RUN` instruction to set the container's hostname.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[serde(transparent)]
pub struct HostnameField(RecipeUnique<String>);

impl RecipeField for HostnameField {
    type Value = Option<String>;

    fn name() -> Option<&'static str> {
        Some("hostname")
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "hostname".to_string())
    }
}

impl Build for HostnameField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        let distro = builder.distro()?;
        if let Some(instruction) = self.value()?.and_then(|h| distro.set_hostname(&h)) {
            builder.push(instruction);
        }
        Ok(())
    }
}
