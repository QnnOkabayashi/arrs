use crate::array::{Array, DType, Shape, TypeAware};

use super::{de, BigEndian};
use std::{fmt, marker::PhantomData, result::Result};

pub struct IdxVisitor<T>
where
    T: TypeAware + BigEndian,
{
    ndims: Option<usize>,
    volume: Option<usize>,
    pd: PhantomData<T>,
}

impl<T> IdxVisitor<T>
where
    T: TypeAware + BigEndian,
{
    pub fn new() -> Self {
        Self {
            ndims: None,
            volume: None,
            pd: PhantomData,
        }
    }
}

impl<'de, T> de::Visitor<'de> for IdxVisitor<T>
where
    T: TypeAware + BigEndian,
{
    type Value = Array<T>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut msg = format!("a magic number with dtype: {}", T::Type::id());
        if let Some(ndims) = self.ndims {
            msg += &format!(", {} dims", ndims);
        }
        if let Some(volume) = self.volume {
            msg += &format!(", {} elements", volume);
        }

        f.write_str(&msg)
    }

    fn visit_bytes<E>(mut self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut strm = v.iter();

        let mut magic = [0u8; 4];
        for byte in magic.iter_mut() {
            *byte = *strm
                .next()
                .ok_or(E::invalid_value(de::Unexpected::Other("eof"), &self))?
        }

        let magic_type = magic[2];
        if magic_type != <T as TypeAware>::Type::id() {
            return Err(E::invalid_value(
                de::Unexpected::Unsigned(magic_type as u64),
                &self,
            ));
        }

        let ndims = magic[3] as usize;
        self.ndims = Some(ndims);
        let mut dims = Vec::with_capacity(ndims);
        for _ in 0..ndims {
            dims.push(
                <i32 as BigEndian>::from_be_bytes(&mut strm)
                    .map_err(|_| E::invalid_value(de::Unexpected::Other("eof"), &self))?
                    as isize,
            );
        }

        let shape = Shape::new(dims);

        let volume = shape.volume() as usize;
        self.volume = Some(volume);
        let mut data = Vec::with_capacity(volume);
        for _ in 0..volume {
            data.push(
                <T as BigEndian>::from_be_bytes(&mut strm)
                    .map_err(|_| E::invalid_value(de::Unexpected::Other("eof"), &self))?,
            );
        }

        Ok(Array::new(shape, data))
    }
}
