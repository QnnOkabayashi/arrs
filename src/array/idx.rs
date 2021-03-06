// only compiles when cfg!(not(no_std))
use crate::array::{ArrResult, ArrayBase, ArrayType, Error, Shape};
use std::fs::File;
use std::io::{Error as IoError, Read, Write};

pub trait IdxType: ArrayType {
    const ID: u8;

    fn read<R: Read>(reader: &mut R) -> ArrResult<Self>;

    fn write<W: Write>(&self, writer: &mut W) -> ArrResult<()>;
}

macro_rules! impl_idxtype {
    { $( ( $inner_type:tt, $size:expr, $id:expr ) ),* } => {
        $(
            impl IdxType for $inner_type {
                const ID: u8 = $id;

                fn read<R: Read>(reader: &mut R) -> ArrResult<Self> {
                    let mut buf = [0; $size];
                    if reader.read(&mut buf)? < $size {
                        return Err(Error::IdxReadUnaccepted);
                    }

                    Ok(Self::from_be_bytes(buf))
                }

                fn write<W: Write>(&self, writer: &mut W) -> ArrResult<()> {
                    let buf = self.to_be_bytes();
                    if writer.write(&buf)? < $size {
                        return Err(Error::IdxWriteUnaccepted);
                    }

                    Ok(())
                }
            }
        )*
    }
}

impl_idxtype! {
    (u8, 1, 0x08),
    (i8, 1, 0x09),
    (i16, 2, 0x0B),
    (i32, 4, 0x0C),
    (f32, 4, 0x0D),
    (f64, 8, 0x0E)
}

impl<T: IdxType, const NDIMS: usize> ArrayBase<T, NDIMS> {
    pub fn from_idx(filename: &'static str) -> ArrResult<Self> {
        let mut file = File::open(filename)?;
        let mut magic = [0; 4];

        if file.read(&mut magic)? < 4 {
            return Err(Error::IdxReadUnaccepted);
        }

        let magic_dtype = magic[2];
        if magic_dtype != T::ID {
            return Err(Error::IdxMismatchDTypeIDs {
                expected: T::ID,
                actual: magic_dtype,
            });
        }

        let magic_ndims = magic[3];
        if magic_ndims != NDIMS as u8 {
            return Err(Error::IdxMismatchNDims {
                expected: NDIMS as u8,
                actual: magic_ndims,
            });
        }

        let mut dims = [0; NDIMS];

        for dim in dims.iter_mut() {
            *dim = <i32 as IdxType>::read(&mut file)? as usize;
        }

        let shape = Shape::new(dims);

        let mut data = Vec::with_capacity(shape.volume());
        for _ in 0..data.capacity() {
            data.push(<T as IdxType>::read(&mut file)?);
        }

        Ok(Self { shape, data })
    }

    pub fn into_idx(&self, filename: &'static str) -> ArrResult<()> {
        unimplemented!(
            "can't read '{}' because this function isn't implemented",
            filename
        )
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::IdxIO {
            message: err.to_string(),
        }
    }
}
