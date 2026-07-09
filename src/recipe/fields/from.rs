use std::path::PathBuf;

use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::BuildError;
use crate::builder::{Build, Builder};
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::path::current_path;
use crate::recipe::unique::RecipeUnique;

/// # From
/// Override the default base image or specify a previous build stage.
///
/// Accepts either:
/// - A container image reference
/// - A relative or absolute path to another config file for multi-stage builds
///
/// When referencing a config file, that config is built as a preceding stage and
/// the current config starts from its output. The `distro` field must be consistent
/// across all stages — it may be defined in either config, but not with different
/// values in both.
#[derive(Debug, Clone, Default, Deserialize, Merge, JsonSchema)]
#[schemars(example = r#"# Image reference
from: ghcr.io/bootcrew/arch-bootc:latest

# Config file reference (multi-stage build)
from: ./base.yaml
"#)]
#[serde(transparent)]
pub struct FromField(RecipeUnique<FromValue>);

/// The resolved value of the `from` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromValue {
    /// A direct container image reference.
    Image(String),
    /// A resolved path to another config file for multi-stage builds.
    Stage(PathBuf),
}

impl RecipeField for FromField {
    type Value = Option<FromValue>;

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |_| "from".to_string())
    }
}

impl Build for FromField {
    fn build(self, builder: &mut Builder) -> Result<(), BuildError> {
        match self.value()? {
            Some(FromValue::Image(image)) => {
                builder.meta_mut().set_base_image(image.clone());
                builder.current_mut().set_from(image);
            }
            Some(FromValue::Stage(path)) => {
                let stage_name = builder.build_stage_from_config(&path)?;
                builder.current_mut().set_from(stage_name);
            }
            None => {
                let image = builder.distro()?.default_image().to_owned();
                builder.meta_mut().set_base_image(image.clone());
                builder.current_mut().set_from(image);
            }
        }

        Ok(())
    }
}

impl JsonSchema for FromValue {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        String::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}

impl<'de> Deserialize<'de> for FromValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let is_path = s.ends_with(".yaml")
            || s.ends_with(".yml")
            || s.starts_with("./")
            || s.starts_with("../")
            || s.starts_with('/')
            || s.starts_with("~");

        if !is_path {
            return Ok(FromValue::Image(s));
        }

        let current = current_path()
            .ok_or_else(|| serde::de::Error::custom("current source path is not set"))?;

        let base_dir = current
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/"));

        let path = base_dir
            .join(PathBuf::from(&s))
            .canonicalize()
            .map_err(|e| {
                serde::de::Error::custom(format!("cannot resolve config path `{s}`: {e}"))
            })?;

        Ok(FromValue::Stage(path))
    }
}
