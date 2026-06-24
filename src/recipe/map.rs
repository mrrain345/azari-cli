use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::path::PathBuf;

use merge::Merge;
use serde::{
    de::{Deserialize, Deserializer, MapAccess, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};

use crate::recipe::error::RecipeError;
use crate::recipe::field::RecipeField;
use crate::recipe::path::current_path;

/// A map field in a recipe file.
///
/// Entries from every source are merged into a single ordered map in source order,
/// preserving the order in which keys were added (imported recipes have precedence).
/// `value()` returns `Err(FieldConflict)` if the same key appears more than once
/// across all sources, including the paths of the conflicting files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeMap<K = String, V = String> {
    values: Vec<(K, V, PathBuf)>,
}

impl<K, V> RecipeMap<K, V> {
    pub fn new(values: Vec<(K, V)>) -> Self {
        let path = current_path().unwrap_or_default();
        Self {
            values: values
                .into_iter()
                .map(|(k, v)| (k, v, path.clone()))
                .collect(),
        }
    }
}

impl<K, V> RecipeField for RecipeMap<K, V>
where
    K: Eq + Hash + fmt::Display,
{
    type Value = Vec<(K, V)>;

    fn name() -> Option<&'static str> {
        None
    }

    /// Returns the merged ordered map across all sources.
    /// Returns `Err(FieldConflict)` if any key appears more than once, with the conflicting paths.
    fn value(self) -> Result<Self::Value, RecipeError> {
        if let Some(error) = self.error() {
            Err(error)
        } else {
            Ok(self.values.into_iter().map(|(k, v, _)| (k, v)).collect())
        }
    }

    fn error(&self) -> Option<RecipeError> {
        let mut key_paths: HashMap<&K, Vec<&PathBuf>> = HashMap::new();

        for (k, _, path) in &self.values {
            key_paths.entry(k).or_default().push(path);
        }

        let errors: Vec<RecipeError> = key_paths
            .into_iter()
            .filter(|(_, paths)| paths.len() > 1)
            .map(|(k, paths)| RecipeError::FieldConflict {
                field: Some(k.to_string()),
                paths: paths.into_iter().cloned().collect(),
            })
            .collect();

        match errors.len() {
            0 => None,
            1 => errors.into_iter().next(),
            _ => Some(RecipeError::Aggregate(errors)),
        }
    }
}

impl<K, V> Merge for RecipeMap<K, V> {
    fn merge(&mut self, other: Self) {
        self.values.extend(other.values);
    }
}

impl<K, V> Default for RecipeMap<K, V> {
    fn default() -> Self {
        Self { values: Vec::new() }
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

impl<'de, K, V> Deserialize<'de> for RecipeMap<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = deserializer.deserialize_option(OptionMapVisitor(PhantomData))?;

        Ok(match opt {
            Some(values) => RecipeMap::new(values),
            None => RecipeMap::default(),
        })
    }
}

impl<K, V> Serialize for RecipeMap<K, V>
where
    K: Serialize + Clone + Eq + Hash + fmt::Display,
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
    use merge::Merge;

    use super::RecipeMap;
    use crate::recipe::error::RecipeError;
    use crate::recipe::field::RecipeField;

    fn pairs(slice: &[(&str, &str)]) -> Vec<(String, String)> {
        slice
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // --- RecipeField::value() ---

    #[test]
    fn default_value_is_empty() {
        let map = RecipeMap::<String, String>::default();
        assert_eq!(map.value().unwrap(), vec![]);
    }

    #[test]
    fn value_returns_pairs_in_insertion_order() {
        let map = RecipeMap::new(pairs(&[("b", "2"), ("a", "1"), ("c", "3")]));
        assert_eq!(
            map.value().unwrap(),
            pairs(&[("b", "2"), ("a", "1"), ("c", "3")])
        );
    }

    #[test]
    fn duplicate_key_within_single_source_is_conflict() {
        let map = RecipeMap::new(pairs(&[("x", "1"), ("x", "2")]));
        assert!(matches!(
            map.value(),
            Err(RecipeError::FieldConflict { .. })
        ));
    }

    // --- merge() ---

    #[test]
    fn merge_unique_keys_preserves_source_order() {
        // "imported has precedence" pattern: imported.merge(current)
        let mut merged = RecipeMap::new(pairs(&[("a", "base"), ("b", "base")]));
        merged.merge(RecipeMap::new(pairs(&[("c", "root")])));
        assert_eq!(
            merged.value().unwrap(),
            pairs(&[("a", "base"), ("b", "base"), ("c", "root")])
        );
    }

    #[test]
    fn merge_duplicate_key_across_sources_is_conflict() {
        let mut merged = RecipeMap::new(pairs(&[("shared", "from-base")]));
        merged.merge(RecipeMap::new(pairs(&[("shared", "from-root")])));
        assert!(matches!(
            merged.value(),
            Err(RecipeError::FieldConflict { .. })
        ));
    }

    #[test]
    fn merge_multiple_duplicate_keys_is_aggregate() {
        let mut merged = RecipeMap::new(pairs(&[("a", "1"), ("b", "1")]));
        merged.merge(RecipeMap::new(pairs(&[("a", "2"), ("b", "2")])));
        assert!(matches!(merged.value(), Err(RecipeError::Aggregate(_))));
    }

    #[test]
    fn merge_with_default_is_identity() {
        let map = RecipeMap::new(pairs(&[("k", "v")]));
        let mut merged = RecipeMap::default();
        merged.merge(map.clone());
        assert_eq!(merged.value().unwrap(), map.value().unwrap());
    }

    // --- Deserialize ---

    #[test]
    fn deserialize_null_yields_default() {
        let map: RecipeMap = serde_saphyr::from_str("~").unwrap();
        assert_eq!(map.value().unwrap(), vec![]);
    }

    #[test]
    fn deserialize_map_yields_ordered_pairs() {
        let map: RecipeMap = serde_saphyr::from_str("b: '2'\na: '1'\nc: '3'").unwrap();
        assert_eq!(
            map.value().unwrap(),
            pairs(&[("b", "2"), ("a", "1"), ("c", "3")])
        );
    }

    // --- Serialize ---

    #[test]
    fn serialize_roundtrip_preserves_order() {
        let original: RecipeMap = serde_saphyr::from_str("b: two\na: one").unwrap();
        let yaml = serde_saphyr::to_string(&original).unwrap();
        let roundtrip: RecipeMap = serde_saphyr::from_str(&yaml).unwrap();
        assert_eq!(original.value().unwrap(), roundtrip.value().unwrap());
    }
}
