use std::marker::PhantomData;

use merge::Merge;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;

/// A receipt field that accepts either a simple (`S`) or complex (`C`) form during deserialization.
///
/// When deserializing, the simple form (`S`) is tried first; on success it is immediately
/// converted to `C` via [`From<S>`]. If the simple form fails, the complex form (`C`) is tried
/// directly. If both fail, only the complex parser's error is reported, so messages always
/// reflect the canonical format rather than the shorthand.
///
/// All merge and conflict-detection semantics are delegated entirely to `C`'s [`Merge`]
/// implementation. [`ReceiptField`] is implemented conditionally when `C: ReceiptField + Default`.
pub struct ReceiptAlt<S, C> {
    inner: Option<C>,
    _marker: PhantomData<fn() -> S>,
}

impl<S, C> ReceiptAlt<S, C> {
    pub fn new(value: C) -> Self {
        Self {
            inner: Some(value),
            _marker: PhantomData,
        }
    }
}

impl<S, C: Clone> Clone for ReceiptAlt<S, C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S, C: std::fmt::Debug> std::fmt::Debug for ReceiptAlt<S, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ReceiptAlt").field(&self.inner).finish()
    }
}

impl<S, C: PartialEq> PartialEq for ReceiptAlt<S, C> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<S, C: Eq> Eq for ReceiptAlt<S, C> {}

impl<S, C> Default for ReceiptAlt<S, C> {
    fn default() -> Self {
        Self {
            inner: None,
            _marker: PhantomData,
        }
    }
}

impl<S, C: Merge> Merge for ReceiptAlt<S, C> {
    fn merge(&mut self, other: Self) {
        match (&mut self.inner, other.inner) {
            (slot @ None, other_inner) => *slot = other_inner,
            (_, None) => {}
            (Some(a), Some(b)) => a.merge(b),
        }
    }
}

impl<S, C> ReceiptField for ReceiptAlt<S, C>
where
    C: ReceiptField + Merge + Default,
{
    type Value = C::Value;

    fn name() -> Option<&'static str> {
        C::name()
    }

    fn value(self) -> Result<Self::Value, ReceiptError> {
        self.inner.unwrap_or_default().value()
    }

    fn error(&self) -> Option<ReceiptError> {
        self.inner.as_ref().and_then(|c| c.error())
    }
}

impl<'de, S, C> Deserialize<'de> for ReceiptAlt<S, C>
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
            return Ok(ReceiptAlt::default());
        }

        // Try the simple form first; on success convert to C immediately via From.
        let de =
            serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(value.clone());
        if let Ok(simple) = S::deserialize(de) {
            return Ok(ReceiptAlt::new(C::from(simple)));
        }

        // Fall back to the complex form; if it also fails, propagate *its* error
        // so that messages reflect the canonical format rather than the shorthand.
        let de = serde_value::ValueDeserializer::<serde_value::DeserializerError>::new(value);
        C::deserialize(de)
            .map(ReceiptAlt::new)
            .map_err(serde::de::Error::custom)
    }
}

impl<S, C: Serialize> Serialize for ReceiptAlt<S, C> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        self.inner.serialize(serializer)
    }
}
