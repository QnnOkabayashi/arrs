/*

Currently putting serde support on the back burner

If CSV parsing is ever used, use fast-float crate for parsing
https://github.com/aldanor/fast-float-rust

*/
use crate::array::{Data, Shape};
use crate::serde_arrs::error::{Error, Result as SdResult};
use crate::serde_arrs::{Array, IdxType, TypeAware};
use de::{DeserializeSeed, MapAccess, Visitor};
use serde::{de, Deserializer};
use std::{fs::File, io::Read, marker::PhantomData};

pub struct IdxDeserializer<'de, T>
where
    T: IdxType<'de>,
{
    file: File,
    marker: PhantomData<&'de T>,
}

impl<'de, T> IdxDeserializer<'de, T>
where
    T: IdxType<'de>,
{
    pub fn from_file(filename: &str) -> SdResult<Self> {
        let file = File::open(filename)?;
        let marker = PhantomData;

        Ok(Self { file, marker })
    }

    fn read_buf(&mut self, buf: &mut [u8]) -> SdResult<()> {
        if self.file.read(buf)? < buf.len() {
            Err(Error::UnexpectedEOF)
        } else {
            Ok(())
        }
    }

    fn read<I>(&mut self) -> SdResult<I>
    where
        I: IdxType<'de>,
    {
        I::read_be_bytes(&mut self.file)
    }

    pub fn parse(&mut self) -> SdResult<Array<T>> {
        let mut magic = [0; 4];

        self.read_buf(&mut magic)?;

        let magic_dtype = magic[2];
        let array_dtype = <T as TypeAware>::ID;
        if magic_dtype != array_dtype {
            return Err(Error::MismatchTypes {
                expected: array_dtype,
                received: magic_dtype,
            });
        }

        let ndims = magic[3] as usize;

        let mut dims = Vec::with_capacity(ndims);
        for _ in 0..ndims {
            dims.push(self.read::<i32>()? as isize);
        }

        let shape = Shape::new(dims);
        let data_len = shape.volume() as usize;

        let mut raw_data = Vec::with_capacity(data_len);
        for _ in 0..data_len {
            raw_data.push(self.read::<T>()?);
        }

        let data = Data::new(raw_data);

        // would've exited earlier and shape and data weren't in sync
        Ok(Array::new(shape, data).unwrap())
    }
}

/*
impl<'de, T: 'de> Deserialize<'de> for Array<T>
where
    T: IdxType<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Shape,
            Data,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "`shape` or `data`")
                    }

                    fn visit_seq<A>(self, seq: A) -> result::Result<Self::Value, A::Error>
                    where
                        A: de::SeqAccess<'de>,
                    {
                        todo!()
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ArrayVisitor<'de, T>(PhantomData<&'de T>)
        where
            T: IdxType<'de>;

        impl<'de, T> de::Visitor<'de> for ArrayVisitor<'de, T>
        where
            T: IdxType<'de>
        {
            type Value = Array<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "struct Array<T>")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut shape_opt = None;
                let mut data_opt = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Shape => match shape_opt {
                            Some(_) => return Err(de::Error::duplicate_field("shape")),
                            None => shape_opt = Some(map.next_value()?),
                        },
                        Field::Data => match data_opt {
                            Some(_) => return Err(de::Error::duplicate_field("data")),
                            None => data_opt = Some(map.next_value()?),
                        },
                    }
                }
                let shape = shape_opt.ok_or(de::Error::missing_field("shape"))?;
                let data = data_opt.ok_or(de::Error::missing_field("data"))?;

                Ok(Array::new(shape, data))
            }
        }

        deserializer.deserialize_struct(
            "Array",
            &["dtype", "shape", "data"],
            ArrayVisitor::<T>(PhantomData),
        )
    }
}

impl<'de> Deserialize<'de> for Shape {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ShapeVisitor;

        impl<'de> Visitor<'de> for ShapeVisitor {
            type Value = Shape;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "A sequence of i32 values representing the shape dimensions"
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut dims = Vec::with_capacity(seq.size_hint().unwrap_or(0));

                while let Some(dim) = seq.next_element::<isize>()? {
                    dims.push(dim);
                }

                Ok(Shape::new(dims))
            }
        }

        deserializer.deserialize_seq(ShapeVisitor)
    }
}

impl<'de, T: 'de> Deserialize<'de> for Data<T>
where
    T: IdxType<'de>,
{
    fn deserialize<D>(deserializer: D) -> result::Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct DataVisitor<'de, T>(PhantomData<&'de T>)
        where
            T: IdxType<'de>;

        impl<'de, T> Visitor<'de> for DataVisitor<'de, T>
        where
            T: IdxType<'de>,
        {
            type Value = Data<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "A sequence of {} values", <T as TypeAware>::Type::new())
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut raw_data = seq
                    .size_hint()
                    .map_or(Vec::new(), |size| Vec::with_capacity(size));

                while let Some(value) = seq.next_element::<T>()? {
                    raw_data.push(value);
                }

                Ok(Data::new(raw_data))
            }
        }

        deserializer.deserialize_seq(DataVisitor(PhantomData))
    }
}

*/

macro_rules! impl_not_implemented {
    { $( $method:ident ),* } => {
        $(
            fn $method<V>(self, _visitor: V) -> SdResult<V::Value>
            where
                V: Visitor<'de>
            {
                Err(Error::NotImplemented {
                    method: stringify!($method),
                })
            }
        )*
    };
}

impl<'de, T> Deserializer<'de> for &mut IdxDeserializer<'de, T>
where
    T: IdxType<'de>,
{
    type Error = Error;

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> SdResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
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
        V: Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_unit_struct",
        })
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> SdResult<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_newtype_struct",
        })
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> SdResult<V::Value>
    where
        V: Visitor<'de>,
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
        V: Visitor<'de>,
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
        V: Visitor<'de>,
    {
        Err(Error::NotImplemented {
            method: "deserialize_enum",
        })
    }
}

impl<'de, T> MapAccess<'de> for IdxDeserializer<'de, T>
where
    T: IdxType<'de>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        // make a deserializer
        seed.deserialize(self).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }
}
