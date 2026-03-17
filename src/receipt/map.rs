use std::fmt;
use std::marker::PhantomData;
use std::path::PathBuf;

use serde::{
    de::{Deserialize, Deserializer, MapAccess, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};

use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::path::current_path;

/// A map field in a receipt file.
///
/// Entries from every source are merged into a single ordered map in source order,
/// preserving the order in which keys were added (imported receipts have precedence).
/// `value()` returns `Err(FieldConflict)` if the same key appears more than once
/// across all sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptMap<K = String, V = String> {
    sources: Vec<PathBuf>,
    values: Vec<Vec<(K, V)>>,
}

impl<K, V> ReceiptMap<K, V> {
    pub fn new(path: PathBuf, values: Vec<(K, V)>) -> Self {
        Self {
            sources: vec![path],
            values: vec![values],
        }
    }
}

impl<K, V> ReceiptField for ReceiptMap<K, V>
where
    K: Eq,
{
    type Value = Vec<(K, V)>;

    /// Returns the merged ordered map across all sources.
    /// Returns `Err(FieldConflict)` if any key appears more than once.
    fn value(self) -> Result<Self::Value, ReceiptError> {
        let flat: Vec<(K, V)> = self.values.into_iter().flatten().collect();
        for i in 0..flat.len() {
            for j in (i + 1)..flat.len() {
                if flat[i].0 == flat[j].0 {
                    return Err(ReceiptError::FieldConflict);
                }
            }
        }
        Ok(flat)
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

impl<K, V> Default for ReceiptMap<K, V> {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            values: Vec::new(),
        }
    }
}

struct PairsVisitor<K, V>(PhantomData<fn() -> (K, V)>);

impl<'de, K, V> Visitor<'de> for PairsVisitor<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = Vec<(K, V)>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a map")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut pairs = Vec::new();
        while let Some((k, v)) = access.next_entry()? {
            pairs.push((k, v));
        }
        Ok(pairs)
    }
}

struct OptionMapVisitor<K, V>(PhantomData<fn() -> (K, V)>);

impl<'de, K, V> Visitor<'de> for OptionMapVisitor<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Value = Option<Vec<(K, V)>>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an optional map")
    }

    fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_some<D2>(self, d: D2) -> Result<Self::Value, D2::Error>
    where
        D2: Deserializer<'de>,
    {
        Ok(Some(d.deserialize_map(PairsVisitor(PhantomData))?))
    }
}

impl<'de, K, V> Deserialize<'de> for ReceiptMap<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = deserializer.deserialize_option(OptionMapVisitor(PhantomData))?;
        let path = current_path().expect("Current source path is not set");

        Ok(match opt {
            Some(values) => ReceiptMap::new(path, values),
            None => ReceiptMap::default(),
        })
    }
}

impl<K, V> Serialize for ReceiptMap<K, V>
where
    K: Serialize + Clone + Eq,
    V: Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let pairs = self.clone().value().map_err(serde::ser::Error::custom)?;
        let mut map = serializer.serialize_map(Some(pairs.len()))?;
        for (k, v) in &pairs {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ReceiptMap;
    use crate::receipt::error::ReceiptError;
    use crate::receipt::field::ReceiptField;
    use crate::receipt::path::SourcePathGuard;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    fn pairs(slice: &[(&str, &str)]) -> Vec<(String, String)> {
        slice
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // --- ReceiptField::value() ---

    #[test]
    fn default_value_is_empty() {
        let map = ReceiptMap::<String, String>::default();
        assert!(map.sources().is_empty());
        assert_eq!(map.value().unwrap(), vec![]);
    }

    #[test]
    fn value_returns_pairs_in_insertion_order() {
        let map = ReceiptMap::new(p("/a.yaml"), pairs(&[("b", "2"), ("a", "1"), ("c", "3")]));
        assert_eq!(
            map.value().unwrap(),
            pairs(&[("b", "2"), ("a", "1"), ("c", "3")])
        );
    }

    #[test]
    fn duplicate_key_within_single_source_is_conflict() {
        let map = ReceiptMap::new(p("/a.yaml"), pairs(&[("x", "1"), ("x", "2")]));
        assert!(matches!(map.value(), Err(ReceiptError::FieldConflict)));
    }

    // --- merge() ---

    #[test]
    fn merge_unique_keys_preserves_source_order() {
        // "imported has precedence" pattern: imported.merge(current)
        let imported = ReceiptMap::new(p("/base.yaml"), pairs(&[("a", "base"), ("b", "base")]));
        let current = ReceiptMap::new(p("/root.yaml"), pairs(&[("c", "root")]));
        let merged = imported.merge(current);
        assert_eq!(
            merged.value().unwrap(),
            pairs(&[("a", "base"), ("b", "base"), ("c", "root")])
        );
    }

    #[test]
    fn merge_duplicate_key_across_sources_is_conflict() {
        let imported = ReceiptMap::new(p("/base.yaml"), pairs(&[("shared", "from-base")]));
        let current = ReceiptMap::new(p("/root.yaml"), pairs(&[("shared", "from-root")]));
        let merged = imported.merge(current);
        assert!(matches!(merged.value(), Err(ReceiptError::FieldConflict)));
    }

    #[test]
    fn merge_tracks_all_sources() {
        let m1 = ReceiptMap::new(p("/a.yaml"), pairs(&[("k1", "v1")]));
        let m2 = ReceiptMap::new(p("/b.yaml"), pairs(&[("k2", "v2")]));
        let merged = m1.merge(m2);
        assert_eq!(merged.sources(), &[p("/a.yaml"), p("/b.yaml")]);
    }

    #[test]
    fn merge_with_default_is_identity() {
        let map = ReceiptMap::new(p("/a.yaml"), pairs(&[("k", "v")]));
        let merged = ReceiptMap::default().merge(map.clone());
        assert_eq!(merged.value().unwrap(), map.value().unwrap());
    }

    // --- Deserialize ---

    #[test]
    fn deserialize_null_yields_default() {
        let _guard = SourcePathGuard::push_path(p("/test.yaml"));
        let map: ReceiptMap = serde_saphyr::from_str("~").unwrap();
        assert!(map.sources().is_empty());
        assert_eq!(map.value().unwrap(), vec![]);
    }

    #[test]
    fn deserialize_map_yields_ordered_pairs() {
        let _guard = SourcePathGuard::push_path(p("/test.yaml"));
        let map: ReceiptMap = serde_saphyr::from_str("b: '2'\na: '1'\nc: '3'").unwrap();
        assert_eq!(
            map.value().unwrap(),
            pairs(&[("b", "2"), ("a", "1"), ("c", "3")])
        );
    }

    #[test]
    fn deserialize_sets_source_path() {
        let _guard = SourcePathGuard::push_path(p("/receipt.yaml"));
        let map: ReceiptMap = serde_saphyr::from_str("key: val").unwrap();
        assert_eq!(map.sources().len(), 1);
    }

    // --- Serialize ---

    #[test]
    fn serialize_roundtrip_preserves_order() {
        let _guard = SourcePathGuard::push_path(p("/test.yaml"));
        let original: ReceiptMap = serde_saphyr::from_str("b: two\na: one").unwrap();
        let yaml = serde_saphyr::to_string(&original).unwrap();
        let _guard2 = SourcePathGuard::push_path(p("/test.yaml"));
        let roundtrip: ReceiptMap = serde_saphyr::from_str(&yaml).unwrap();
        assert_eq!(original.value().unwrap(), roundtrip.value().unwrap());
    }
}
