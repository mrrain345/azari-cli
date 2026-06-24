use std::path::PathBuf;

use merge::Merge;
use schemars::JsonSchema;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::recipe::error::RecipeError;
use crate::recipe::field::RecipeField;
use crate::recipe::path::current_path;

impl<T: JsonSchema> JsonSchema for RecipeUnique<T> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        T::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        T::json_schema(generator)
    }
}

/// A recipe field where only one source may define a value.
/// Multiple sources defining it with different values results in a [`RecipeError::FieldConflict`].
/// Multiple sources defining the same value are silently deduplicated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeUnique<T = String> {
    values: Vec<T>,
    paths: Vec<PathBuf>,
}

impl<T> RecipeUnique<T> {
    pub fn new(value: T) -> Self {
        let path = current_path().unwrap_or_default();
        Self {
            values: vec![value],
            paths: vec![path],
        }
    }
}

impl<T: PartialEq> RecipeField for RecipeUnique<T> {
    type Value = Option<T>;

    fn name() -> Option<&'static str> {
        None
    }

    /// Returns the unique value if defined by at most one source,
    /// or `Err(FieldConflict)` if multiple sources define different values.
    fn value(self) -> Result<Self::Value, RecipeError> {
        if let Some(error) = self.error() {
            Err(error)
        } else {
            Ok(self.values.into_iter().next())
        }
    }

    fn error(&self) -> Option<RecipeError> {
        if self.values.len() <= 1 {
            return None;
        }

        let first = &self.values[0];

        if self.values.iter().all(|v| v == first) {
            None
        } else {
            let paths = self.paths.clone();
            Some(RecipeError::FieldConflict { field: None, paths })
        }
    }
}

impl<T> Merge for RecipeUnique<T> {
    fn merge(&mut self, other: Self) {
        self.values.extend(other.values);
        self.paths.extend(other.paths);
    }
}

impl<T> Default for RecipeUnique<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            paths: Vec::new(),
        }
    }
}

impl<'de, T> Deserialize<'de> for RecipeUnique<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<T>::deserialize(deserializer)?;

        Ok(match opt {
            Some(value) => RecipeUnique::new(value),
            None => RecipeUnique::default(),
        })
    }
}

impl<T> Serialize for RecipeUnique<T>
where
    T: PartialEq + Clone + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.clone()
            .value()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
