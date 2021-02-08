use crate::array::{DType, TypeAware};

use super::{de, Endianess, Shape};
use std::{fmt, marker, result};

pub(super) struct MagicNumberVisitor {
    dtype: u8,
}

impl MagicNumberVisitor {
    pub fn new(dtype: u8) -> Self {
        Self { dtype }
    }
}

impl<'de> de::Visitor<'de> for MagicNumberVisitor {
    type Value = usize;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "a byte array with at least 4 bytes, where the third byte is {:#X}",
            self.dtype
        )
    }

    fn visit_bytes<E>(self, b: &[u8]) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        // check that a magic number can be read
        if b.len() < 4 {
            return Err(E::invalid_length(b.len(), &self));
        }
        // check for matching dtype
        if b[2] != self.dtype {
            return Err(E::invalid_value(de::Unexpected::Bytes(&b[2..]), &self));
        }
        Ok(b[3] as usize)
    }
}

pub(super) struct ShapeVisitor {
    ndims: usize,
}

impl ShapeVisitor {
    pub fn new(ndims: usize) -> Self {
        Self { ndims }
    }
}

impl<'de> de::Visitor<'de> for ShapeVisitor {
    type Value = Shape;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "a byte array containing at least {} 32-bit integers",
            self.ndims
        )
    }

    fn visit_bytes<E>(self, b: &[u8]) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        if b.len() < 4 * self.ndims {
            return Err(E::invalid_length(b.len(), &self));
        }

        Ok(Shape::new(
            (0..self.ndims)
                .map(|chunk| {
                    let offset = chunk * 4;
                    let dim = <i32 as Endianess>::from_be_bytes(&b[offset..offset + 4]);

                    dim as isize
                })
                .collect(),
        ))
    }
}

pub(super) struct DataVisitor<T>
where
    T: TypeAware + Endianess,
{
    len: usize,
    pd: marker::PhantomData<T>,
}

impl<T> DataVisitor<T>
where
    T: TypeAware + Endianess,
{
    pub fn new(len: isize) -> Self {
        Self {
            len: len as usize,
            pd: marker::PhantomData,
        }
    }
}

impl<'de, T> de::Visitor<'de> for DataVisitor<T>
where
    T: TypeAware + Endianess,
{
    type Value = Vec<T>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "a byte array containing at least {} bytes for {} {}'s",
            self.len * <T as TypeAware>::Type::bytes(),
            self.len,
            <T as TypeAware>::Type::new()
        )
    }

    fn visit_bytes<E>(self, b: &[u8]) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        if b.len() < self.len {
            return Err(E::invalid_length(b.len(), &self));
        }

        Ok((0..self.len)
            .map(|chunk| {
                let size = <T as TypeAware>::Type::bytes();
                let offset = chunk * size;

                <T as Endianess>::from_be_bytes(&b[offset..offset + size])
            })
            .collect())
    }
}
