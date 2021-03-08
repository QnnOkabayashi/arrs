// only compiles when cfg!(not(no_std))
use crate::array::{ArrResult, ArrayBase, Error};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

macro_rules! impl_idxtype {
    { $inner_type:ty, $size:expr, $id:expr } => {
        impl<const NDIMS: usize> ArrayBase<$inner_type, NDIMS> {
            pub fn from_idx(filename: &'static str) -> ArrResult<Self> {
                let mut reader = BufReader::new(File::open(filename)?);

                let mut magic = [0; 4];
                if reader.read(&mut magic)? < 4 {
                    return Err(Error::IdxReadUnaccepted);
                }

                let magic_dtype = magic[2];
                if magic_dtype != $id {
                    return Err(Error::IdxMismatchDTypeIDs {
                        expected: $id,
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
                    let mut bytes = [0; 4];
                    if reader.read(&mut bytes)? < 4 {
                        return Err(Error::IdxReadUnaccepted);
                    }
                    *dim = i32::from_be_bytes(bytes) as usize;
                }

                let mut data = Vec::with_capacity(dims.iter().product());
                for _ in 0..data.capacity() {
                    let mut bytes = [0; $size];
                    if reader.read(&mut bytes)? < $size {
                        return Err(Error::IdxReadUnaccepted);
                    }
                    data.push(<$inner_type>::from_be_bytes(bytes));
                }

                Ok(Self { dims, data })
            }

            pub fn into_idx(self, filename: &'static str) -> ArrResult<()> {
                let mut writer = BufWriter::new(File::create(filename)?);

                writer.write(&[0, 0, $id, NDIMS as u8])?;

                for &dim in self.dims.iter() {
                    writer.write(&(dim as i32).to_be_bytes())?;
                }

                for &value in self.data.iter() {
                    writer.write(&value.to_be_bytes())?;
                }

                Ok(())
            }
        }
    }
}

impl_idxtype! { u8, 1, 0x08 }
impl_idxtype! { i8, 1, 0x09 }
impl_idxtype! { i16, 2, 0x0B }
impl_idxtype! { i32, 4, 0x0C }
impl_idxtype! { f32, 4, 0x0D }
impl_idxtype! { f64, 8, 0x0E }

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IdxIO {
            message: err.to_string(),
        }
    }
}
