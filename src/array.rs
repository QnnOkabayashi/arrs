mod dense;
mod error;
mod shape;
mod traits;
use self::traits::IdxType;
pub use dense::Array as DenseArray;
pub use error::{ArrResult, Error};
use shape::BroadcastInstruction;
pub use shape::{Shape, ShapeBase};
pub use traits::{Broadcastable, PartialView, TypeAware};

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
impl<T: IdxType> ArrayBase<T> {
    pub fn from_idx(filename: &'static str) -> ArrResult<Self> {
        use std::fs::File;
        use std::io::Read;

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

// struct SparseArray; // dynamic access patterns
