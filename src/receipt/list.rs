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
    type Value<'a>
        = Vec<&'a T>
    where
        T: 'a;

    /// Returns the merged list of values across all sources.
    fn value(&self) -> Result<Vec<&T>, ReceiptError> {
        Ok(self.values.iter().flat_map(|v| v.iter()).collect())
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

    use super::ReceiptList;
    use crate::receipt::field::ReceiptField;
    use crate::receipt::path::SourcePathGuard;

    // Test value()

    #[test]
    fn default_list_is_empty() {
        let list: ReceiptList = ReceiptList::default();
        assert!(list.value().unwrap().is_empty());
    }

    #[test]
    fn list_with_value_returns_value() {
        let list = ReceiptList::new(
            "receipt.yaml".into(),
            vec!["foo".to_string(), "bar".to_string()],
        );
        let value = list.value().unwrap();
        assert_eq!(value[0], "foo");
        assert_eq!(value[1], "bar");
        assert_eq!(value.len(), 2);
    }

    #[test]
    fn list_with_empty_vec_is_empty() {
        let list = ReceiptList::<String>::new("receipt.yaml".into(), vec![]);
        assert!(list.value().unwrap().is_empty());
    }

    #[test]
    fn multiple_sources_merges_values() {
        let list: ReceiptList<String> = ReceiptList {
            sources: vec!["a.yaml".into(), "b.yaml".into()],
            values: vec![
                vec!["foo".to_string()],
                vec!["bar".to_string(), "baz".to_string()],
            ],
        };
        let value = list.value().unwrap();
        assert_eq!(value.len(), 3);
        assert_eq!(value[0], "foo");
        assert_eq!(value[1], "bar");
        assert_eq!(value[2], "baz");
    }

    #[test]
    fn multiple_sources_all_empty_is_empty() {
        let list: ReceiptList<String> = ReceiptList {
            sources: vec!["a.yaml".into(), "b.yaml".into()],
            values: vec![vec![], vec![]],
        };
        assert!(list.value().unwrap().is_empty());
    }

    // Test merge()

    #[test]
    fn merge_two_empty_lists() {
        let a: ReceiptList<String> = ReceiptList::default();
        let b: ReceiptList<String> = ReceiptList::default();
        let merged = a.merge(b);
        assert!(merged.value().unwrap().is_empty());
    }

    #[test]
    fn merge_empty_with_values() {
        let a: ReceiptList<String> = ReceiptList::default();
        let b = ReceiptList::new("b.yaml".into(), vec!["foo".to_string(), "bar".to_string()]);
        let merged = a.merge(b);
        let value = merged.value().unwrap();
        assert_eq!(value.len(), 2);
        assert_eq!(value[0], "foo");
        assert_eq!(value[1], "bar");
    }

    #[test]
    fn merge_combines_values_in_order() {
        let a = ReceiptList::new("a.yaml".into(), vec!["foo".to_string()]);
        let b = ReceiptList::new("b.yaml".into(), vec!["bar".to_string(), "baz".to_string()]);
        let merged = a.merge(b);
        let value = merged.value().unwrap();
        assert_eq!(value.len(), 3);
        assert_eq!(value[0], "foo");
        assert_eq!(value[1], "bar");
        assert_eq!(value[2], "baz");
    }

    #[test]
    fn merge_preserves_source_order() {
        let a = ReceiptList::new("a.yaml".into(), vec!["foo".to_string()]);
        let b = ReceiptList::new("b.yaml".into(), vec!["bar".to_string()]);
        let merged = a.merge(b);
        let sources = merged.sources();
        assert_eq!(sources[0], PathBuf::from("a.yaml"));
        assert_eq!(sources[1], PathBuf::from("b.yaml"));
    }

    // Test Deserialize

    #[test]
    #[should_panic]
    fn deserialize_without_context_should_panic() {
        let _result: Result<ReceiptList<String>, _> = serde_saphyr::from_str("- hello");
    }

    #[test]
    fn deserialize_with_context() {
        let path = PathBuf::from("receipt.yaml");
        let _guard = SourcePathGuard::push_path(path.clone());
        let list: ReceiptList<String> = serde_saphyr::from_str("- foo\n- bar\n").unwrap();
        let value = list.value().unwrap();
        assert_eq!(value[0], "foo");
        assert_eq!(value[1], "bar");
        assert_eq!(list.sources()[0], path);
    }

    #[test]
    fn deserialize_null_with_context() {
        let path = PathBuf::from("receipt.yaml");
        let _guard = SourcePathGuard::push_path(path.clone());
        let list: ReceiptList<String> = serde_saphyr::from_str("~").unwrap();
        assert!(list.value().unwrap().is_empty());
    }
}
