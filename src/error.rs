use core::fmt::{self, Display};
use core::result::Result;

pub type ArrResult<T> = Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    MatMul {
        rows_a: usize,
        cols_b: usize,
    },
    Broadcast {
        dims1: Vec<usize>,
        dims2: Vec<usize>,
    },
    ShapeZeroDims,
    ShapeZeroLenDim {
        dims: Vec<usize>,
    },
    ShapeDataMisalignment {
        volume: usize,
        len: usize,
    },
    DerankIndexOutOfBounds {
        len: usize,
        index: usize,
    },
    ReadNDim {
        ndims: usize,
    },
    Derank1D,
    SliceZeroStep,
    SliceZeroWidth {
        index: usize,
    },
    SliceNonConverging {
        start: usize,
        end: usize,
        step: isize,
    },
    SliceStartOutOfBounds {
        start: usize,
        len: usize,
    },
    SliceEndOutOfBounds {
        end: usize,
        len: usize,
    },
    SliceTooLarge {
        size: usize,
        len: usize,
    },
    IdxIO {
        // io::Error doesn't impl PartialEq, which is annoying
        message: String,
    },
    IdxReadUnaccepted,
    IdxWriteUnaccepted,
    IdxMismatchDTypeIDs {
        expected: u8,
        actual: u8,
    },
    IdxMismatchNDims {
        expected: u8,
        actual: u8,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            MatMul { rows_a, cols_b } => {
                write!(
                    f,
                    "cannot matrix multiply: first has {} rows, second has {} cols",
                    rows_a, cols_b
                )
            }
            Broadcast { dims1, dims2 } => {
                write!(
                    f,
                    "operands could not be broadcast together with shapes {:?}, {:?}",
                    dims1, dims2
                )
            }
            ShapeZeroDims => {
                write!(f, "shape cannot be constructed with 0 dims")
            }
            ShapeZeroLenDim { dims } => {
                write!(f, "shape cannot have a dim of width 0, received {:?}", dims)
            }
            ShapeDataMisalignment {
                volume: shape_volume,
                len: data_len,
            } => {
                write!(
                    f,
                    "shape volume is {}, but {} elements were provided",
                    shape_volume, data_len
                )
            }
            DerankIndexOutOfBounds { len, index } => {
                write!(f, "cannot derank at index {} when len is {}", index, len)
            }
            ReadNDim { ndims } => {
                write!(f, "cannot read an individual value from an array with more than 1 dim, has {} dims", ndims)
            }
            Derank1D => {
                write!(
                    f,
                    "cannot slice down to a smaller dimension from a 1D array"
                )
            }
            SliceZeroStep => {
                write!(f, "slice cannot have 0 step")
            }
            SliceZeroWidth { index } => {
                write!(
                    f,
                    "slice cannot have 0 size, start: {i}, stop: {i}",
                    i = index
                )
            }
            SliceNonConverging { start, end, step } => {
                write!(
                    f,
                    "slice doesn't converge with start: {}, end: {}, step: {}",
                    start, end, step
                )
            }
            SliceStartOutOfBounds { start, len } => {
                write!(f, "slice start: {} is greater than len: {}", start, len)
            }
            SliceEndOutOfBounds { end, len } => {
                write!(f, "slice end: {} is greater than len: {}", end, len)
            }
            SliceTooLarge { size, len } => {
                write!(f, "slice size: {} is greater than len: {}", size, len)
            }
            IdxIO { message } => f.write_str(&message),
            IdxReadUnaccepted => {
                write!(f, "reader no longer providing bytes")
            }
            IdxWriteUnaccepted => {
                write!(f, "writer no longer accepting bytes")
            }
            IdxMismatchDTypeIDs { expected, actual } => {
                write!(f, "expected dtype ID: {}, found ID: {}", expected, actual)
            }
            IdxMismatchNDims { expected, actual } => {
                write!(f, "expected {} dims, found {} dims", expected, actual)
            }
        }
    }
}
