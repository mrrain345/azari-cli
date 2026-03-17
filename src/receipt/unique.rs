use std::path::PathBuf;

use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::path::current_path;

/// A receipt field where only one source may define a value.
/// Multiple sources defining it results in a [`ReceiptError::FieldConflict`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptUnique<T = String> {
    sources: Vec<PathBuf>,
    values: Vec<T>,
}

impl<T> ReceiptUnique<T> {
    pub fn new(path: PathBuf, value: T) -> Self {
        Self {
            sources: vec![path],
            values: vec![value],
        }
    }
}

impl<T> ReceiptField for ReceiptUnique<T> {
    type Value = Option<T>;

    /// Returns `Err(FieldConflict)` if more than one source defined this field.
    fn value(self) -> Result<Self::Value, ReceiptError> {
        if self.values.len() > 1 {
            return Err(ReceiptError::FieldConflict);
        }

        Ok(self.values.into_iter().next())
    }

    fn sources(&self) -> &[PathBuf] {
        &self.sources
    }

    fn merge(self, other: Self) -> Self {
        let mut sources = self.sources;
        let mut values = self.values;
        sources.extend(other.sources);
        values.extend(other.values);
        Self { sources, values }
    }
}

impl<T> Default for ReceiptUnique<T> {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<'de, T> Deserialize<'de> for ReceiptUnique<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<T>::deserialize(deserializer)?;
        let path = current_path().expect("Current source path is not set");

        Ok(match opt {
            Some(value) => ReceiptUnique::new(path, value),
            None => ReceiptUnique::default(),
        })
    }
}

impl<T> Serialize for ReceiptUnique<T>
where
    T: Serialize + Clone,
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
