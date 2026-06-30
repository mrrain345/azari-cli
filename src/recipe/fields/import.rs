use std::path::PathBuf;

use merge::Merge;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::recipe::RecipeError;
use crate::recipe::field::RecipeField;
use crate::recipe::path::current_path;

/// # Import
/// Import additional config files.
///
/// Paths may be absolute or relative to the current config file.
///
/// **Example:**
/// ```yaml
/// import:
///   - common.yaml
///   - ../shared/base.yaml
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportField {
    imports: Vec<PathBuf>,
    errors: Vec<(PathBuf, String)>,
}

impl ImportField {
    /// Creates import state from one source config and raw `import` entries.
    /// Relative paths are resolved against the source config directory.
    pub fn new(source: PathBuf, imports: Vec<PathBuf>) -> Self {
        let source = source.canonicalize().unwrap_or(source);

        let base_dir = source
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/"));

        let mut resolved = Vec::with_capacity(imports.len());
        let mut errors = Vec::new();

        for p in imports {
            let path = if p.is_absolute() { p } else { base_dir.join(p) };
            match path.canonicalize() {
                Ok(path) => resolved.push(path),
                Err(source_error) => {
                    let message = match source_error.kind() {
                        std::io::ErrorKind::NotFound => {
                            format!("missing file `{}`", path.display())
                        }
                        _ => format!("failed to read `{}`: {source_error}", path.display()),
                    };
                    errors.push((source.clone(), message));
                }
            }
        }

        Self {
            imports: resolved,
            errors,
        }
    }

    pub fn take_imports(&mut self) -> Vec<PathBuf> {
        std::mem::take(&mut self.imports)
    }
}

impl JsonSchema for ImportField {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        Vec::<String>::schema_name()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let mut schema = Vec::<String>::json_schema(generator);
        schema.insert("title".to_owned(), "Import".into());
        schema.insert(
            "description".to_owned(),
            "Import additional config files.".into(),
        );
        schema.insert(
            "examples".to_owned(),
            serde_json::json!([r#"import:
  - common.yaml
  - ../shared/base.yaml"#]),
        );

        schema
    }
}

impl RecipeField for ImportField {
    type Value = Vec<PathBuf>;

    /// Imports (empty when full load completes).
    fn value(self) -> Result<Self::Value, RecipeError> {
        Ok(self.imports)
    }

    fn error(&self) -> Option<RecipeError> {
        let errors: Vec<RecipeError> = self
            .errors
            .iter()
            .map(|(path, message)| RecipeError::FieldError {
                path: path.clone(),
                field: "import".to_string(),
                message: message.clone(),
            })
            .collect();

        match errors.len() {
            0 => None,
            1 => errors.into_iter().next(),
            _ => Some(RecipeError::Aggregate(errors)),
        }
    }
}

impl IntoIterator for ImportField {
    type Item = PathBuf;
    type IntoIter = std::vec::IntoIter<PathBuf>;

    fn into_iter(self) -> Self::IntoIter {
        self.imports.into_iter()
    }
}

impl Merge for ImportField {
    fn merge(&mut self, other: Self) {
        self.imports.extend(other.imports);
        self.errors.extend(other.errors);
    }
}

impl<'de> Deserialize<'de> for ImportField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let imports = Option::<Vec<PathBuf>>::deserialize(deserializer)?.unwrap_or_default();
        let source = current_path()
            .ok_or_else(|| serde::de::Error::custom("current source path is not set"))?;
        Ok(ImportField::new(source, imports))
    }
}

impl Serialize for ImportField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Option::<()>::None.serialize(serializer)
    }
}
