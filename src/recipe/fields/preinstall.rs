use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::list::RecipeList;

/// # Preinstall
/// Execute shell commands at the beginning of the build process.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[schemars(example = r#"preinstall:
  - mkdir -p /opt/custom"#)]
#[serde(transparent)]
pub struct PreinstallField(RecipeList<String>);

impl RecipeField for PreinstallField {
    type Value = Vec<String>;

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "preinstall".to_string())
    }
}

impl Build for PreinstallField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        for command in self.value()? {
            builder.push(format!("RUN {command}"));
        }
        Ok(())
    }
}
