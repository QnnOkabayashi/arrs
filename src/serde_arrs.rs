mod de;
mod error;
use crate::array::{Array1, TypeAware};
use de::IdxDeserializer;
use error::{Error, Result as SdResult};
use serde::de::Deserialize;
use std::io::Read;

pub trait IdxType<'de>: TypeAware + Deserialize<'de> {
    fn read_be_bytes<R: Read>(reader: &mut R) -> SdResult<Self>;

    // probably takes a Writer
    // fn write_be_bytes<W: Write>(&self, writer: &mut W) -> SdResult<()>;
}

macro_rules! impl_idxtype {
    { $( $type:ty: $size:expr ),* } => {
        $(
            impl<'de> IdxType<'de> for $type {
                fn read_be_bytes<R: Read>(reader: &mut R) -> SdResult<Self> {
                    let mut buf = [0; $size];
                    if reader.read(&mut buf)? < $size {
                        Err(Error::UnexpectedEOF)
                    } else {
                        Ok(Self::from_be_bytes(buf))
                    }
                }
            }
        )*
    }
}

impl_idxtype! {
    u8: 1,
    i8: 1,
    i16: 2,
    i32: 4,
    f32: 4,
    f64: 8
}

pub fn from_idx<'de, T: 'de>(filename: &str) -> SdResult<Array1<T>>
where
    T: IdxType<'de>,
{
    let mut deserializer = IdxDeserializer::<T>::from_file(filename)?;
    deserializer.parse()
    // <Array<T> as Deserialize>::deserialize(&mut deserializer)
}
