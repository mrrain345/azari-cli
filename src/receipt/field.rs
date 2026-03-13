use std::path::PathBuf;

use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::receipt::error::ReceiptError;
use crate::receipt::path::current_path;

/// A single optional field in a receipt file.
///
/// Semantically represents `Option<T>`: the field is either absent or holds one
/// value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptField<T = String> {
    sources: Vec<(PathBuf, T)>,
}

impl<T> ReceiptField<T> {
    /// Creates a field with a single value from a known source path.
    pub fn new(path: PathBuf, value: T) -> Self {
        Self {
            sources: vec![(path, value)],
        }
    }

    /// Returns the value of the field, if present.
    ///
    /// Returns `ReceiptError::FieldConflict` if the field has been defined in
    /// more than one source file (i.e. a merge conflict is present).
    pub fn value(&self) -> Result<Option<&T>, ReceiptError> {
        if self.sources.len() > 1 {
            Err(ReceiptError::FieldConflict)
        } else {
            Ok(self.sources.first().map(|(_, v)| v))
        }
    }

    /// Returns all `(path, value)` pairs that contributed to this field.
    pub fn sources(&self) -> &[(PathBuf, T)] {
        &self.sources
    }
}

impl<T> Default for ReceiptField<T> {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
        }
    }
}

impl<'de, T> Deserialize<'de> for ReceiptField<T>
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
            Some(value) => ReceiptField::new(path, value),
            None => ReceiptField::default(),
        })
    }
}

impl<T> Serialize for ReceiptField<T>
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

    use super::ReceiptField;
    use crate::receipt::error::ReceiptError;
    use crate::receipt::path::SourcePathGuard;

    // Test value()

    #[test]
    fn default_field_is_empty() {
        let field: ReceiptField = ReceiptField::default();
        assert!(field.value().unwrap().is_none());
    }

    #[test]
    fn field_with_value_returns_value() {
        let field = ReceiptField::new("receipt.yaml".into(), "hello".to_string());
        assert_eq!(field.value().unwrap().map(String::as_str), Some("hello"));
    }

    #[test]
    fn multiple_sources_returns_conflict() {
        let field: ReceiptField<String> = ReceiptField {
            sources: vec![
                ("a.yaml".into(), "first".to_string()),
                ("b.yaml".into(), "second".to_string()),
            ],
        };
        assert!(matches!(field.value(), Err(ReceiptError::FieldConflict)));
    }

    // Test Deserialize

    #[test]
    #[should_panic]
    fn deserialize_without_context_should_panic() {
        let _result: Result<ReceiptField<String>, _> = serde_saphyr::from_str("hello");
    }

    #[test]
    fn deserialize_with_context() {
        let path = PathBuf::from("receipt.yaml");
        let _guard = SourcePathGuard::push_path(path.clone());
        let field: ReceiptField<String> = serde_saphyr::from_str("hello").unwrap();
        assert_eq!(field.value().unwrap().map(String::as_str), Some("hello"));
        assert_eq!(field.sources[0].0, path);
    }
}
