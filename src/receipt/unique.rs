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
    type Value<'a>
        = Option<&'a T>
    where
        T: 'a;

    /// Returns `Err(FieldConflict)` if more than one source defined this field.
    fn value(&self) -> Result<Option<&T>, ReceiptError> {
        if self.values.len() > 1 {
            Err(ReceiptError::FieldConflict)
        } else {
            Ok(self.values.first())
        }
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
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ReceiptUnique;
    use crate::receipt::error::ReceiptError;
    use crate::receipt::field::ReceiptField;
    use crate::receipt::path::SourcePathGuard;

    // Test value()

    #[test]
    fn default_field_is_empty() {
        let field: ReceiptUnique = ReceiptUnique::default();
        assert!(field.value().unwrap().is_none());
    }

    #[test]
    fn field_with_value_returns_value() {
        let field = ReceiptUnique::new("receipt.yaml".into(), "hello".to_string());
        assert_eq!(field.value().unwrap().map(String::as_str), Some("hello"));
    }

    #[test]
    fn multiple_sources_returns_conflict() {
        let field: ReceiptUnique<String> = ReceiptUnique {
            sources: vec!["a.yaml".into(), "b.yaml".into()],
            values: vec!["first".to_string(), "second".to_string()],
        };
        assert!(matches!(field.value(), Err(ReceiptError::FieldConflict)));
    }

    // Test merge()

    #[test]
    fn merge_two_empty_fields() {
        let a: ReceiptUnique<String> = ReceiptUnique::default();
        let b: ReceiptUnique<String> = ReceiptUnique::default();
        assert!(a.merge(b).value().unwrap().is_none());
    }

    #[test]
    fn merge_empty_with_value() {
        let a: ReceiptUnique<String> = ReceiptUnique::default();
        let b = ReceiptUnique::new("b.yaml".into(), "hello".to_string());
        let merged = a.merge(b);
        assert_eq!(merged.value().unwrap().map(String::as_str), Some("hello"));
    }

    #[test]
    fn merge_value_with_empty() {
        let a = ReceiptUnique::new("a.yaml".into(), "hello".to_string());
        let b: ReceiptUnique<String> = ReceiptUnique::default();
        let merged = a.merge(b);
        assert_eq!(merged.value().unwrap().map(String::as_str), Some("hello"));
    }

    #[test]
    fn merge_two_values_creates_conflict() {
        let a = ReceiptUnique::new("a.yaml".into(), "first".to_string());
        let b = ReceiptUnique::new("b.yaml".into(), "second".to_string());
        assert!(matches!(
            a.merge(b).value(),
            Err(ReceiptError::FieldConflict)
        ));
    }

    #[test]
    fn merge_preserves_source_order() {
        let a = ReceiptUnique::new("a.yaml".into(), "first".to_string());
        let b = ReceiptUnique::new("b.yaml".into(), "second".to_string());
        let sources = a.merge(b).sources().to_vec();
        assert_eq!(sources[0], PathBuf::from("a.yaml"));
        assert_eq!(sources[1], PathBuf::from("b.yaml"));
    }

    // Test Deserialize

    #[test]
    #[should_panic]
    fn deserialize_without_context_should_panic() {
        let _result: Result<ReceiptUnique<String>, _> = serde_saphyr::from_str("hello");
    }

    #[test]
    fn deserialize_with_context() {
        let path = PathBuf::from("receipt.yaml");
        let _guard = SourcePathGuard::push_path(path.clone());
        let field: ReceiptUnique<String> = serde_saphyr::from_str("hello").unwrap();
        assert_eq!(field.value().unwrap().map(String::as_str), Some("hello"));
        assert_eq!(field.sources()[0], path);
    }
}
