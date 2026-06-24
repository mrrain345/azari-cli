use std::collections::HashMap;

use merge::Merge;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;

// ---- IniAny ----

/// A wrapper for any value that can be serialized/deserialized by Serde.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IniAny(pub serde_value::Value);

impl JsonSchema for IniAny {
    fn schema_name() -> Cow<'static, str> {
        "IniAny".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        // IniAny can be any JSON value, so use a permissive empty schema
        schemars::json_schema!({})
    }
}

// ---- IniMulti ----

/// A multi-value field that accepts either a single value or a list.
///
/// Deserializes from `T` or `Vec<T>`.
/// Serializes as a sequence, which the INI serializer renders as repeated
/// `Key=value` lines with the same key.
///
/// # Example
/// ```yaml
/// # Single form:
/// after: network.target
/// # List form:
/// after:
///   - network.target
///   - bar.service
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct IniMulti<T>(Vec<T>);

impl<T: JsonSchema> JsonSchema for IniMulti<T> {
    fn schema_name() -> Cow<'static, str> {
        format!("IniMulti_{}", T::schema_name()).into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        #[derive(schemars::JsonSchema)]
        #[allow(dead_code)]
        #[schemars(untagged)]
        enum IniMultiSchema<T> {
            Single(T),
            Many(Vec<T>),
        }

        IniMultiSchema::<T>::json_schema(generator)
    }
}

impl<T> Default for IniMulti<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> IniMulti<T> {
    pub fn new(values: Vec<T>) -> Self {
        Self(values)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> Merge for IniMulti<T> {
    fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for IniMulti<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value =
            serde_value::Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;

        if matches!(
            value,
            serde_value::Value::Option(None) | serde_value::Value::Unit
        ) {
            return Ok(IniMulti::default());
        }

        // Try single T first (simple form).
        let de =
            serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(value.clone());
        if let Ok(single) = T::deserialize(de) {
            return Ok(IniMulti(vec![single]));
        }

        // Fall back to Vec<T>.
        let de = serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(value);
        Vec::<T>::deserialize(de)
            .map(IniMulti)
            .map_err(serde::de::Error::custom)
    }
}

impl<T: Serialize> Serialize for IniMulti<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for item in &self.0 {
            seq.serialize_element(item)?;
        }
        seq.end()
    }
}

// ---- IniExtra ----

/// A map of less-common fields intended for use with `#[serde(flatten)]`.
///
/// Values are stored as untyped [`IniAny`] so that integers,
/// booleans, and strings from YAML are all accepted without a fixed schema.
///
/// Merges by extending; later sources win on duplicate keys.
#[derive(Debug, Default, PartialEq, JsonSchema)]
pub struct IniExtra(pub(crate) HashMap<String, IniAny>);

impl IniExtra {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Merge for IniExtra {
    fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl<'de> Deserialize<'de> for IniExtra {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map = HashMap::<String, serde_value::Value>::deserialize(deserializer)?;
        Ok(IniExtra(
            map.into_iter().map(|(k, v)| (k, IniAny(v))).collect(),
        ))
    }
}

impl Serialize for IniExtra {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut sorted: Vec<_> = self.0.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        let mut map = serializer.serialize_map(Some(sorted.len()))?;
        for (k, v) in sorted {
            // Keys are passed as-is; the INI serializer applies kebab_to_pascal.
            map.serialize_entry(k.as_str(), &v.0)?;
        }
        map.end()
    }
}
