use merge::Merge;
use serde::{Deserialize, Serialize};

use crate::recipe::error::RecipeError;
use crate::recipe::field::RecipeField;

/// A list field in a recipe file.
///
/// Items from every source are merged into a single flat list in source order.
/// Multiple sources defining this field is not a conflict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeList<T = String>(Vec<T>);

impl<T> RecipeList<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self(values)
    }
}

impl<T> RecipeField for RecipeList<T> {
    type Value = Vec<T>;

    fn name() -> Option<&'static str> {
        None
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        Ok(self.0)
    }

    fn error(&self) -> Option<RecipeError> {
        None
    }
}

impl<T> Merge for RecipeList<T> {
    fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl<T> Default for RecipeList<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> From<Vec<T>> for RecipeList<T> {
    fn from(values: Vec<T>) -> Self {
        Self(values)
    }
}

impl<T> From<T> for RecipeList<T> {
    fn from(value: T) -> Self {
        Self(vec![value])
    }
}
