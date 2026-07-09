use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::BuildError;
use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::unique::RecipeUnique;

/// # Hostname
/// Set the hostname.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[serde(transparent)]
pub struct HostnameField(RecipeUnique<String>);

impl RecipeField for HostnameField {
    type Value = Option<String>;

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "hostname".to_string())
    }
}

impl Build for HostnameField {
    fn build(self, builder: &mut Builder) -> Result<(), BuildError> {
        if let Some(hostname) = self.value()? {
            builder.distro()?.set_hostname(builder, &hostname);
        }

        Ok(())
    }
}
