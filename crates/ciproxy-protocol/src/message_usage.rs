//! Read a raw JSON field without buffering it with the other message fields.

use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

use serde::de::value::{
    BorrowedStrDeserializer, MapAccessDeserializer, SeqAccessDeserializer, UnitDeserializer,
};
use serde::de::{DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;

/// Define a tagged message once for serialization and for strict field decoding.
macro_rules! usage_message {
    ($(#[$attribute:meta])* pub enum $name:ident $variants:tt) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        pub enum $name $variants

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                type Message = $name;

                #[derive(serde::Deserialize)]
                #[serde(remote = "Message", tag = "type", rename_all = "snake_case")]
                enum Fields $variants

                #[derive(serde::Deserialize)]
                #[serde(transparent)]
                struct Wire(#[serde(with = "Fields")] Message);

                let (Wire(mut message), raw) =
                    $crate::message_usage::split_raw_field(deserializer, "usage")?;
                if let Some(raw) = raw {
                    message.set_raw_usage(&raw);
                }
                Ok(message)
            }
        }
    };
}

pub(crate) use usage_message;

/// Decode a JSON map with a null in place of each occurrence of a named field.
/// Keep the last raw value separately. The caller still sees every field key.
pub(crate) fn split_raw_field<'de, D, T>(
    deserializer: D,
    field: &str,
) -> Result<(T, Option<Box<RawValue>>), D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserializer.deserialize_any(SplitField {
        field,
        output: PhantomData,
    })
}

struct SplitField<'field, T> {
    field: &'field str,
    output: PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for SplitField<'_, T> {
    type Value = (T, Option<Box<RawValue>>);

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a tagged message or usage object")
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        let mut raw = None;
        let decoded = T::deserialize(MapAccessDeserializer::new(RawFieldMap {
            map,
            field: self.field,
            raw: &mut raw,
            read_raw: false,
        }))?;
        Ok((decoded, raw))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        T::deserialize(SeqAccessDeserializer::new(seq)).map(|decoded| (decoded, None))
    }
}

struct RawFieldMap<'field, A> {
    map: A,
    field: &'field str,
    raw: &'field mut Option<Box<RawValue>>,
    read_raw: bool,
}

#[derive(Deserialize)]
struct Key<'key>(#[serde(borrow)] Cow<'key, str>);

impl<'de, A: MapAccess<'de>> MapAccess<'de> for RawFieldMap<'_, A> {
    type Error = A::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        let Some(Key(key)) = self.map.next_key()? else {
            return Ok(None);
        };
        self.read_raw = key == self.field;
        match key {
            Cow::Borrowed(key) => seed.deserialize(BorrowedStrDeserializer::new(key)),
            Cow::Owned(key) => seed.deserialize(key.into_deserializer()),
        }
        .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        if self.read_raw {
            *self.raw = Some(self.map.next_value()?);
            seed.deserialize(UnitDeserializer::new())
        } else {
            self.map.next_value_seed(seed)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.map.size_hint()
    }
}
