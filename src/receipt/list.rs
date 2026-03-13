use std::path::PathBuf;

use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::receipt::path::current_path;

/// A list field in a receipt file.
///
/// Semantically represents `Vec<T>`: items from every source
/// are merged into a single flat list in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptList<T = String> {
    sources: Vec<(PathBuf, Vec<T>)>,
}

impl<T> ReceiptList<T> {
    /// Creates a list field with values from a known source path.
    pub fn new(path: PathBuf, values: Vec<T>) -> Self {
        Self {
            sources: vec![(path, values)],
        }
    }

    /// Returns the merged list of values across all sources.
    pub fn value(&self) -> Vec<&T> {
        self.sources.iter().flat_map(|(_, v)| v.iter()).collect()
    }

    /// Returns all `(path, values)` pairs that contributed to this list.
    pub fn sources(&self) -> &[(PathBuf, Vec<T>)] {
        &self.sources
    }
}

impl<T> Default for ReceiptList<T> {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
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
        self.value().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ReceiptList;
    use crate::receipt::path::SourcePathGuard;

    // Test value()

    #[test]
    fn default_list_is_empty() {
        let list: ReceiptList = ReceiptList::default();
        assert!(list.value().is_empty());
    }

    #[test]
    fn list_with_value_returns_value() {
        let list = ReceiptList::new(
            "receipt.yaml".into(),
            vec!["foo".to_string(), "bar".to_string()],
        );
        let value = list.value();
        assert_eq!(value[0], "foo");
        assert_eq!(value[1], "bar");
        assert_eq!(value.len(), 2);
    }

    #[test]
    fn list_with_empty_vec_is_empty() {
        let list = ReceiptList::<String>::new("receipt.yaml".into(), vec![]);
        assert!(list.value().is_empty());
    }

    #[test]
    fn multiple_sources_merges_values() {
        let list: ReceiptList<String> = ReceiptList {
            sources: vec![
                ("a.yaml".into(), vec!["foo".to_string()]),
                ("b.yaml".into(), vec!["bar".to_string(), "baz".to_string()]),
            ],
        };
        let value = list.value();
        assert_eq!(value.len(), 3);
        assert_eq!(value[0], "foo");
        assert_eq!(value[1], "bar");
        assert_eq!(value[2], "baz");
    }

    #[test]
    fn multiple_sources_all_empty_is_empty() {
        let list: ReceiptList<String> = ReceiptList {
            sources: vec![("a.yaml".into(), vec![]), ("b.yaml".into(), vec![])],
        };
        assert!(list.value().is_empty());
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
        let value = list.value();
        assert_eq!(value[0], "foo");
        assert_eq!(value[1], "bar");
        assert_eq!(list.sources[0].0, path);
    }

    #[test]
    fn deserialize_null_with_context() {
        let path = PathBuf::from("receipt.yaml");
        let _guard = SourcePathGuard::push_path(path.clone());
        let list: ReceiptList<String> = serde_saphyr::from_str("~").unwrap();
        assert!(list.value().is_empty());
    }
}
