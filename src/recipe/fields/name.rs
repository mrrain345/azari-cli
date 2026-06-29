use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::unique::RecipeUnique;

/// # Name
/// Human-readable OS name.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[schemars(example = "name: Azari Workstation")]
#[serde(transparent)]
pub struct NameField(RecipeUnique<String>);

impl RecipeField for NameField {
    type Value = Option<String>;

    fn name() -> Option<&'static str> {
        Some("name")
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "name".to_string())
    }
}

impl Build for NameField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        if let Some(name) = self.value()? {
            let version = builder.version().map(str::to_owned);
            let pretty = match &version {
                Some(v) => format!("{name} {v}"),
                None => name.clone(),
            };

            let mut cmd = Vec::new();
            cmd.push(os_release_sed("NAME", &name));

            if let Some(v) = &version {
                cmd.push(os_release_sed("VERSION", v));
            }

            cmd.push(os_release_sed("PRETTY_NAME", &pretty));
            builder.push(format!("RUN {}", cmd.join(" && ")));
            builder.set_name(pretty);
        }
        Ok(())
    }
}

/// Produces a `sed` command that updates `KEY="value"` in
/// `os-release` in-place, or appends it if the key is not present.
fn os_release_sed(key: &str, value: &str) -> String {
    let value_e = escape(value);
    format!(
        "sed -i '/^{key}=/{{s/.*/{key}=\"{value_e}\"/;:a;n;ba}};$a{key}=\"{value_e}\"' /etc/os-release /usr/lib/os-release"
    )
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
