use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::list::RecipeList;

/// Field for `preinstall` and `postinstall` keys.
///
/// Each entry is a shell command that is emitted as its own `RUN` instruction.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[serde(transparent)]
pub struct InstallField(RecipeList<String>);

impl RecipeField for InstallField {
    type Value = Vec<String>;

    fn name() -> Option<&'static str> {
        None
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "install".to_string())
    }
}

impl Build for InstallField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        for command in self.value()? {
            builder.push(format!("RUN {command}"));
        }
        Ok(())
    }
}
