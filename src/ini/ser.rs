use std::fmt::Display;

use serde::ser::{self, Impossible, Serialize};

// ---- Public API ----

/// Serializes a value as a systemd-style INI unit file.
///
/// The input must be a struct or map whose top-level fields become `[Section]`
/// headers. Each section's fields become `Key=Value` lines. Multi-value fields
/// (sequences) are rendered as repeated `Key=Value` lines with the same key.
/// `None` fields are omitted. Boolean values are rendered as `yes`/`no`.
///
/// Field names are converted from `kebab-case` to `PascalCase` when writing
/// (`exec-start` → `ExecStart`, `OOM-score-adjust` → `OOMScoreAdjust`).
pub fn to_string<T: Serialize>(value: &T) -> Result<String, Error> {
    let mut file = FileSerializer {
        output: String::new(),
        first_section: true,
        pending_section: None,
    };
    value.serialize(&mut file)?;
    Ok(file.output)
}

// ---- Error ----

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
    #[error("section or map key must be a string")]
    NonStringKey,
    #[error("top-level INI value must be a struct or map, not a scalar")]
    TopLevelScalar,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

// ---- File-level serializer ----

struct FileSerializer {
    output: String,
    first_section: bool,
    pending_section: Option<String>,
}

impl FileSerializer {
    fn write_section(&mut self, name: &str, content: &str) {
        if !self.first_section {
            self.output.push('\n');
        }
        self.first_section = false;
        self.output.push('[');
        self.output.push_str(name);
        self.output.push_str("]\n");
        self.output.push_str(content);
    }

    fn serialize_section<T: Serialize + ?Sized>(
        &mut self,
        name: &str,
        value: &T,
    ) -> Result<(), Error> {
        let mut section = SectionSerializer {
            output: String::new(),
            pending_key: None,
        };
        value.serialize(&mut section)?;
        if !section.output.is_empty() {
            self.write_section(name, &section.output);
        }
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut FileSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = &'a mut FileSerializer;
    type SerializeStruct = &'a mut FileSerializer;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _: bool) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_i8(self, _: i8) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_i16(self, _: i16) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_i32(self, _: i32) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_i64(self, _: i64) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_u8(self, _: u8) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_u16(self, _: u16) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_u32(self, _: u32) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_u64(self, _: u64) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_f32(self, _: f32) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_f64(self, _: f64) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_char(self, _: char) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_str(self, _: &str) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_none(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Impossible<(), Error>, Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_tuple(self, _: usize) -> Result<Impossible<(), Error>, Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::TopLevelScalar)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::TopLevelScalar)
    }
}

impl ser::SerializeStruct for &mut FileSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.serialize_section(&kebab_to_pascal(key), value)
    }

    fn skip_field(&mut self, _: &'static str) -> Result<(), Error> {
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl ser::SerializeMap for &mut FileSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        let mut kc = KeyCollector::default();
        key.serialize(&mut kc)?;
        self.pending_section = Some(kebab_to_pascal(&kc.key));
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let name = self.pending_section.take().unwrap_or_default();
        self.serialize_section(&name, value)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

// ---- Section-level serializer ----

struct SectionSerializer {
    output: String,
    pending_key: Option<String>,
}

impl<'a> ser::Serializer for &'a mut SectionSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = &'a mut SectionSerializer;
    type SerializeStruct = &'a mut SectionSerializer;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _: bool) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_i8(self, _: i8) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_i16(self, _: i16) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_i32(self, _: i32) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_i64(self, _: i64) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_u8(self, _: u8) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_u16(self, _: u16) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_u32(self, _: u32) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_u64(self, _: u64) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_f32(self, _: f32) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_f64(self, _: f64) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_char(self, _: char) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_str(self, _: &str) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_none(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Impossible<(), Error>, Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_tuple(self, _: usize) -> Result<Impossible<(), Error>, Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self, Error> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::Custom(
            "section value must be a struct or map".into(),
        ))
    }
}

impl ser::SerializeStruct for &mut SectionSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        let pascal = kebab_to_pascal(key);
        let entry = EntrySerializer {
            output: &mut self.output,
            key: pascal,
        };
        value.serialize(entry)
    }

    fn skip_field(&mut self, _: &'static str) -> Result<(), Error> {
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

/// Implements `SerializeMap` so that `#[serde(flatten)]` fields work:
/// serde wraps the section state in `FlatMapSerializer<&mut SectionSerializer>`
/// which requires `&mut SectionSerializer: SerializeMap`.
impl ser::SerializeMap for &mut SectionSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        let mut kc = KeyCollector::default();
        key.serialize(&mut kc)?;
        self.pending_key = Some(kebab_to_pascal(&kc.key));
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let key = self.pending_key.take().unwrap_or_default();
        let entry = EntrySerializer {
            output: &mut self.output,
            key,
        };
        value.serialize(entry)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

// ---- Entry-level serializer ----

struct EntrySerializer<'a> {
    output: &'a mut String,
    key: String,
}

struct MultiEntryState<'a> {
    output: &'a mut String,
    key: String,
}

struct InlineMapState<'a> {
    output: &'a mut String,
    pending_key: Option<String>,
}

fn push_entry(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push('=');
    output.push_str(value);
    output.push('\n');
}

impl<'a> ser::Serializer for EntrySerializer<'a> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = MultiEntryState<'a>;
    type SerializeTuple = MultiEntryState<'a>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = InlineMapState<'a>;
    type SerializeStruct = InlineMapState<'a>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        push_entry(self.output, &self.key, if v { "yes" } else { "no" });
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        push_entry(self.output, &self.key, &v.to_string());
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        push_entry(self.output, &self.key, &v.to_string());
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        push_entry(self.output, &self.key, &v.to_string());
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        push_entry(self.output, &self.key, &v.to_string());
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        push_entry(self.output, &self.key, &v.to_string());
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        push_entry(self.output, &self.key, &v.to_string());
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        push_entry(self.output, &self.key, &v.to_string());
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        push_entry(self.output, &self.key, &v.to_string());
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        push_entry(self.output, &self.key, &fmt_float(v as f64));
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        push_entry(self.output, &self.key, &fmt_float(v));
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<(), Error> {
        push_entry(self.output, &self.key, &v.to_string());
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        push_entry(self.output, &self.key, normalize_float_str(v));
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Error> {
        Err(Error::Custom(
            "bytes cannot be serialized as an INI value".into(),
        ))
    }
    fn serialize_none(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Error> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        push_entry(self.output, &self.key, variant);
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), Error> {
        Err(Error::Custom(
            "enum variants cannot be serialized as an INI value".into(),
        ))
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<MultiEntryState<'a>, Error> {
        Ok(MultiEntryState {
            output: self.output,
            key: self.key,
        })
    }
    fn serialize_tuple(self, len: usize) -> Result<MultiEntryState<'a>, Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::Custom(
            "tuple structs cannot be serialized as an INI value".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::Custom(
            "tuple variants cannot be serialized as an INI value".into(),
        ))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<InlineMapState<'a>, Error> {
        Ok(InlineMapState {
            output: self.output,
            pending_key: None,
        })
    }
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<InlineMapState<'a>, Error> {
        self.serialize_map(Some(len))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::Custom(
            "struct variants cannot be serialized as an INI value".into(),
        ))
    }
}

// ---- MultiEntryState: emits Key=v for each element ----

impl<'a> ser::SerializeSeq for MultiEntryState<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let entry = EntrySerializer {
            output: self.output,
            key: self.key.clone(),
        };
        value.serialize(entry)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for MultiEntryState<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let entry = EntrySerializer {
            output: self.output,
            key: self.key.clone(),
        };
        value.serialize(entry)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

// ---- InlineMapState: inlines map entries directly (no outer key) ----

impl<'a> ser::SerializeMap for InlineMapState<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        let mut kc = KeyCollector::default();
        key.serialize(&mut kc)?;
        self.pending_key = Some(kebab_to_pascal(&kc.key));
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        let key = self.pending_key.take().unwrap_or_default();
        let entry = EntrySerializer {
            output: self.output,
            key,
        };
        value.serialize(entry)
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for InlineMapState<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        let entry = EntrySerializer {
            output: self.output,
            key: kebab_to_pascal(key),
        };
        value.serialize(entry)
    }

    fn skip_field(&mut self, _: &'static str) -> Result<(), Error> {
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

// ---- KeyCollector: serializes a key as a String ----

#[derive(Default)]
struct KeyCollector {
    key: String,
}

impl ser::Serializer for &mut KeyCollector {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _: bool) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_i8(self, _: i8) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_i16(self, _: i16) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_i32(self, _: i32) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_i64(self, _: i64) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_u8(self, _: u8) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_u16(self, _: u16) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_u32(self, _: u32) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_u64(self, _: u64) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_f32(self, _: f32) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_f64(self, _: f64) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.key = v.to_string();
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.key = v.to_owned();
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_none(self) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.key = variant.to_owned();
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Impossible<(), Error>, Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_tuple(self, _: usize) -> Result<Impossible<(), Error>, Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Impossible<(), Error>, Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Impossible<(), Error>, Error> {
        Err(Error::NonStringKey)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Impossible<(), Error>, Error> {
        Err(Error::NonStringKey)
    }
}

// ---- Helpers ----

/// Converts a kebab-case string to PascalCase.
///
/// Each `-`-delimited word has its first character uppercased; the rest of
/// the word is left as-is so that mixed-case acronyms are preserved:
/// `exec-start-pre` → `ExecStartPre`
/// `OOM-score-adjust` → `OOMScoreAdjust`
pub(crate) fn kebab_to_pascal(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Formats a float for use as an INI value.
///
/// Rust renders `f64::INFINITY` as `"inf"` and `f64::NAN` as `"NaN"`, but
/// systemd (and most INI parsers) expect `"infinity"` and `"nan"` respectively.
fn fmt_float(v: f64) -> String {
    if v.is_infinite() {
        if v.is_sign_negative() {
            "-infinity".to_owned()
        } else {
            "infinity".to_owned()
        }
    } else if v.is_nan() {
        "nan".to_owned()
    } else {
        v.to_string()
    }
}

/// Normalises YAML float literal strings to their systemd-compatible form.
///
/// serde-saphyr represents YAML infinity/nan as their canonical YAML 1.1
/// string forms (`.inf`, `+.inf`, `-.inf`, `.nan`) when deserialising into
/// `serde_value::Value::String`. This converts them back to the plain words
/// that systemd understands.
fn normalize_float_str(s: &str) -> &str {
    match s {
        ".inf" | "+.inf" | ".Inf" | "+.Inf" | ".INF" | "+.INF" => "infinity",
        "-.inf" | "-.Inf" | "-.INF" => "-infinity",
        ".nan" | ".NaN" | ".NAN" => "nan",
        other => other,
    }
}
