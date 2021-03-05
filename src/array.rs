mod dense;
mod error;
mod shape;
mod traits;
pub use dense::Array as DenseArray;
pub use error::{ArrResult, Error};
use shape::BroadcastInstruction;
pub use shape::{Shape, ShapeBase};
pub use traits::{MultiDimensional, PartialView, TypeAware};

pub struct ArrayBase<T: TypeAware> {
    shape_base: ShapeBase,
    data: Vec<T>,
}

impl<T: TypeAware> ArrayBase<T> {
    pub fn new_checked(dims: Vec<usize>, data: Vec<T>) -> ArrResult<Self> {
        let shape_base = ShapeBase::new_checked(dims)?;

        if shape_base.total_volume() != data.len() {
            return Err(Error::ShapeDataMisalignment {
                shape_volume: shape_base.total_volume(),
                data_len: data.len(),
            });
        }

        Ok(Self { shape_base, data })
    }
}

#[cfg(not(no_std))]
mod idx {
    use crate::array::{ArrResult, ArrayBase, Error, ShapeBase, TypeAware};
    use std::fs::File;
    use std::io::{Read, Write, Error as IoError};

    pub trait IdxType: TypeAware {
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

    impl<T: IdxType> ArrayBase<T> {
        pub fn from_idx(filename: &'static str) -> ArrResult<Self> {
            let mut file = File::open(filename)?;
            let mut magic = [0; 4];
            if file.read(&mut magic)? < 4 {
                return Err(Error::IdxReadUnaccepted);
            }

            let magic_dtype = magic[2];
            if magic_dtype != T::ID {
                return Err(Error::MismatchDTypeIDs {
                    dtype1: T::ID,
                    dtype2: magic_dtype,
                });
            }

            let ndims = magic[3] as usize;

            let mut dims = Vec::with_capacity(ndims);
            for _ in 0..ndims {
                dims.push(<i32 as IdxType>::read(&mut file)? as usize);
            }

            let shape_base = ShapeBase::new_checked(dims)?;
            let volume = shape_base.total_volume();

            let mut data = Vec::with_capacity(volume);
            for _ in 0..volume {
                data.push(<T as IdxType>::read(&mut file)?);
            }

            Ok(Self { shape_base, data })
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
}