use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::list::RecipeList;

/// Field for the `packages` key.
///
/// Merges package lists from all imported recipes and emits a single
/// distro-specific `RUN` install instruction.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[serde(transparent)]
pub struct PackagesField(RecipeList<String>);

impl RecipeField for PackagesField {
    type Value = Vec<String>;

    fn name() -> Option<&'static str> {
        Some("packages")
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "packages".to_string())
    }
}

impl Build for PackagesField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        let distro = builder.distro()?;
        let packages = self.value()?;
        if !packages.is_empty() {
            let refs: Vec<&str> = packages.iter().map(String::as_str).collect();
            if let Some(instruction) = distro.install_packages(&refs) {
                builder.push(instruction);
            }
        }
        Ok(())
    }
}
