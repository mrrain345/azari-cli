use std::marker::PhantomData;

use merge::Merge;
use schemars::JsonSchema;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::recipe::error::RecipeError;
use crate::recipe::field::RecipeField;

/// A recipe field that accepts either a simple (`S`) or complex (`C`) form during deserialization.
///
/// When deserializing, the simple form (`S`) is tried first; on success it is immediately
/// converted to `C` via [`From<S>`]. If the simple form fails, the complex form (`C`) is tried
/// directly. If both fail, only the complex parser's error is reported, so messages always
/// reflect the canonical format rather than the shorthand.
///
/// All merge and conflict-detection semantics are delegated entirely to `C`'s [`Merge`]
/// implementation. [`RecipeField`] is implemented conditionally when `C: RecipeField + Default`.
pub struct RecipeAlt<S, C> {
    inner: Option<C>,
    _marker: PhantomData<fn() -> S>,
}

impl<S, C> RecipeAlt<S, C> {
    pub fn new(value: C) -> Self {
        Self {
            inner: Some(value),
            _marker: PhantomData,
        }
    }
}

impl<S, C: Clone> Clone for RecipeAlt<S, C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S, C: std::fmt::Debug> std::fmt::Debug for RecipeAlt<S, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RecipeAlt").field(&self.inner).finish()
    }
}

impl<S, C: PartialEq> PartialEq for RecipeAlt<S, C> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<S, C: Eq> Eq for RecipeAlt<S, C> {}

impl<S, C> Default for RecipeAlt<S, C> {
    fn default() -> Self {
        Self {
            inner: None,
            _marker: PhantomData,
        }
    }
}

impl<S, C: Merge> Merge for RecipeAlt<S, C> {
    fn merge(&mut self, other: Self) {
        match (&mut self.inner, other.inner) {
            (slot @ None, other_inner) => *slot = other_inner,
            (_, None) => {}
            (Some(a), Some(b)) => a.merge(b),
        }
    }
}

impl<S, C> RecipeField for RecipeAlt<S, C>
where
    C: RecipeField + Merge + Default,
{
    type Value = C::Value;

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.inner.unwrap_or_default().value()
    }

    fn error(&self) -> Option<RecipeError> {
        self.inner.as_ref().and_then(|c| c.error())
    }
}

impl<'de, S, C> Deserialize<'de> for RecipeAlt<S, C>
where
    S: Deserialize<'de>,
    C: Deserialize<'de> + From<S> + Merge,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Buffer the raw input as a format-agnostic Value so we can retry
        // deserialization for each form without consuming the deserializer twice.
        let value =
            serde_value::Value::deserialize(deserializer).map_err(serde::de::Error::custom)?;

        // Absent / null → treat as missing field.
        if matches!(
            value,
            serde_value::Value::Option(None) | serde_value::Value::Unit
        ) {
            return Ok(RecipeAlt::default());
        }

        // Try the simple form first; on success convert to C immediately via From.
        let de =
            serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(value.clone());
        if let Ok(simple) = S::deserialize(de) {
            return Ok(RecipeAlt::new(C::from(simple)));
        }

        // Fall back to the complex form; if it also fails, propagate *its* error
        // so that messages reflect the canonical format rather than the shorthand.
        let de = serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(value);
        C::deserialize(de)
            .map(RecipeAlt::new)
            .map_err(serde::de::Error::custom)
    }
}

impl<S, C: Serialize> Serialize for RecipeAlt<S, C> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<S: JsonSchema, C: JsonSchema> JsonSchema for RecipeAlt<S, C> {
    fn inline_schema() -> bool {
        S::inline_schema() && C::inline_schema()
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        format!("RecipeAlt_{}_{}", S::schema_name(), C::schema_name()).into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        #[derive(schemars::JsonSchema)]
        #[allow(dead_code)]
        #[schemars(untagged)]
        enum AltSchema<S, C> {
            Simple(S),
            Complex(C),
        }

        AltSchema::<S, C>::json_schema(generator)
    }
}
