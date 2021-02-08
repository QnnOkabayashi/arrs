mod visitor;
use super::error::{Error, Result};
use super::{Array, DType, Endianess, Shape, TypeAware};
use serde::{de, Deserialize};
use std::result;
use visitor::{DataVisitor, MagicNumberVisitor, ShapeVisitor};

pub fn from_bytes<'a, A>(input: Vec<u8>) -> Result<A>
where
    A: Deserialize<'a>,
{
    let mut deserializer = IdxDeserializer::from_bytes(&input);
    let arr = A::deserialize(&mut deserializer);
    if deserializer.is_done() {
        arr
    } else {
        Err(Error::TrailingBytes)
    }
}

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
    T: TypeAware + Endianess,
{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, <D as de::Deserializer<'de>>::Error>
    where
        D: de::Deserializer<'de>,
    {
        // do everything through deserialize_struct?
        let ndims = deserializer
            .deserialize_bytes(MagicNumberVisitor::new(<T as TypeAware>::Type::id()))?;
        let shape = deserializer.deserialize_bytes(ShapeVisitor::new(ndims))?;
        let data = deserializer.deserialize_bytes(DataVisitor::new(shape.volume()))?;

        Ok(Array::new(shape, data))
        // let magic = {
        //     let read_magic: ResultV<u8> = (0..mem::size_of::<i32>())
        //         .map(|_| deserializer.deserialize_u8(U8Visitor))
        //         .collect();
        //     read_magic?
        // };
        //     let array_type = <T as TypeAware>::Type::id();
        //     let magic_type = magic[2];
        //     if array_type != magic_type {
        //         return Err(Error::MismatchTypes {
        //             expected: array_type,
        //             received: magic_type,
        //         });
        //     }
        //     let shape = Shape::new({
        //         let ndims = magic[3];
        //         let read_dims: ResultV<isize> = (0..ndims)
        //             .map(|_| self.read_next::<i32>().and_then(|x| Ok(x as isize)))
        //             .collect();
        //         read_dims?
        //     });
        //     let data = {
        //         let read_data: ResultV<T> = (0..shape.volume())
        //             .map(|_| self.read_next::<T>())
        //             .collect();
        //         read_data?
        //     };
        //     Ok(Array::new(shape, data))
    }
}

macro_rules! impl_deserialize {
    { $( $deserialize_b:ident ),* } => {
        $(
            fn $deserialize_b<V>(self, _visitor: V) -> Result<V::Value>
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

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(self.input)
    }

    impl_deserialize! {
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
        deserialize_byte_buf,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_unit_struct",
        })
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_newtype_struct",
        })
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
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
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_tuple_struct",
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_struct",
        })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_enum",
        })
    }
}
