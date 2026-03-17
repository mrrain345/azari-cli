use std::path::PathBuf;

use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::path::current_path;

/// A list field in a receipt file.
///
/// Items from every source are merged into a single flat list in source order.
/// Multiple sources defining this field is not a conflict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptList<T = String> {
    sources: Vec<PathBuf>,
    values: Vec<Vec<T>>,
}

impl<T> ReceiptList<T> {
    /// Creates a list field from a known source path.
    pub fn new(path: PathBuf, values: Vec<T>) -> Self {
        Self {
            sources: vec![path],
            values: vec![values],
        }
    }
}

impl<T> ReceiptField for ReceiptList<T> {
    type Value = Vec<T>;

    /// Returns the merged list of values across all sources.
    fn value(self) -> Result<Self::Value, ReceiptError> {
        Ok(self.values.into_iter().flatten().collect())
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

impl<T> Default for ReceiptList<T> {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            values: Vec::new(),
        }
    }
}

impl<'de, T> Deserialize<'de> for ReceiptList<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<Vec<T>>::deserialize(deserializer)?;
        let path = current_path().expect("Current source path is not set");

        Ok(match opt {
            Some(values) => ReceiptList::new(path, values),
            None => ReceiptList::default(),
        })
    }
}

impl<T> Serialize for ReceiptList<T>
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
