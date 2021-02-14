mod visitor;
use super::error::{Error, Result as SdResult};
use super::{Array, BigEndian, TypeAware};
use serde::{de, Deserialize};
use std::result;
use visitor::IdxVisitor;


pub struct IdxDeserializer<'de> {
    input: &'de [u8],
}

impl<'de> IdxDeserializer<'de> {
    pub fn is_done(&mut self) -> bool {
        self.input.len() == 0
    }

    pub fn from_bytes(input: &'de [u8]) -> Self {
        Self { input }
    }
}

impl<'de, T> de::Deserialize<'de> for Array<T>
where
    T: TypeAware + BigEndian,
{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, <D as de::Deserializer<'de>>::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_struct("Array", &["shape", "data"], IdxVisitor::<T>::new())
    }
}

macro_rules! impl_not_implemented {
    { $( $deserialize_b:ident ),* } => {
        $(
            fn $deserialize_b<V>(self, _visitor: V) -> SdResult<V::Value>
            where
                V: de::Visitor<'de>
            {
                Err(Error::NotImplemented {
                    method: stringify!($deserialized_b),
                })
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for &mut IdxDeserializer<'de> {
    type Error = Error;

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> SdResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(self.input)
    }

    impl_not_implemented! {
        deserialize_bool,
        deserialize_i8,
        deserialize_u8,
        deserialize_char,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_f32,
        deserialize_f64,
        deserialize_any,
        deserialize_str,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> SdResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_unit_struct",
        })
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> SdResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_newtype_struct",
        })
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> SdResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_tuple",
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> SdResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_tuple_struct",
        })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> SdResult<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_enum",
        })
    }
}
