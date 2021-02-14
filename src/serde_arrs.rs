mod de;
mod error;
mod ser;
use super::array::{Array, TypeAware};
use error::{Error, Result as SdResult};
use ser::Serializer;
use de::IdxDeserializer;
use serde::Deserialize;
use std::convert::TryInto;
use std::mem;
use std::slice::Iter;

pub trait BigEndian: Sized {
    fn from_be_bytes(strm: &mut Iter<u8>) -> SdResult<Self>;

    // fn to_be_bytes(&self) -> [u8];
}

macro_rules! impl_endianess {
    { one_byte: [ $( $one_type:ty ),* ], n_bytes: [ $( $n_type:ty ),* ] } => {
        // one_byte
        $(
            impl BigEndian for $one_type {
                fn from_be_bytes(strm: &mut Iter<u8>) -> SdResult<Self> {
                    if let Some(byte) = strm.next() {
                        Ok(*byte as Self)
                    } else {
                        Err(Error::UnexpectedEOF)
                    }
                }
            }
        )*
        // n_bytes
        $(
            impl BigEndian for $n_type {
                fn from_be_bytes(bytes: &mut Iter<u8>) -> SdResult<Self> {
                    let nbytes = mem::size_of::<$n_type>();
                    let mut buf = Vec::with_capacity(nbytes);
                    for _ in 0..nbytes {
                        buf.push(*bytes.next().ok_or(Error::UnexpectedEOF)?)
                    }

                    Ok(Self::from_be_bytes(buf.try_into().unwrap()))
                }
            }
        )*
    }
}

impl_endianess! {
    one_byte: [
        u8, i8
    ],
    n_bytes: [
        i16, i32, f32, f64
    ]
}

#[test]
fn compile() {}

pub fn from_bytes<T>(input: Vec<u8>) -> SdResult<Array<T>>
where
    T: TypeAware + BigEndian,
{
    let mut deserializer = IdxDeserializer::from_bytes(&input);
    let arr = <Array::<T> as Deserialize>::deserialize(&mut deserializer);
    arr
    // if deserializer.is_done() {
    //     arr
    // } else {
    //     Err(Error::TrailingBytes)
    // }
}
