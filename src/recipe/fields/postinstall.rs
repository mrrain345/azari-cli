use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::list::RecipeList;

/// # Postinstall
/// Execute shell commands at the end of the build process.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[schemars(example = r#"postinstall:
  - rm -rf /tmp/* /var/tmp/* /var/cache/*"#)]
#[serde(transparent)]
pub struct PostinstallField(RecipeList<String>);

impl RecipeField for PostinstallField {
    type Value = Vec<String>;

    fn name() -> Option<&'static str> {
        None
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "postinstall".to_string())
    }
}

impl Build for PostinstallField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        for command in self.value()? {
            builder.push(format!("RUN {command}"));
        }
        Ok(())
    }
}
