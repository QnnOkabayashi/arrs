use crate::array::{Shape, Shape1};
use std::{
    fmt::{self, Display},
    result::Result,
};

pub type ArrResult<T> = Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Cast {
        lhs: Shape1,
        rhs: Shape1,
    }, // remove once new shapes work
    Broadcast {
        dims1: Vec<usize>,
        dims2: Vec<usize>,
    },
    Reshape {
        initial: Shape1,
        target: Shape1,
    },
    ShapeZeroDims,
    ShapeZeroLenDim {
        dims: Vec<usize>,
    },
    ShapeDataMisalignment {
        shape_volume: usize,
        data_len: usize,
    },
    ShapeDataMisalignment1 {
        shape: Shape1,
        data_len: usize,
    }, // remove once new shapes work
    FromIdxFile {
        filename: &'static str,
    },
    DerankIndexOutOfBounds {
        len: usize,
        index: usize,
    },
    ReadNDim {
        ndims: usize,
    },
    Derank1D,
    SliceZeroWidth {
        index: usize,
    },
    SliceStopBeforeStart {
        start: usize,
        stop: usize,
    },
    SliceStopPastEnd {
        stop: usize,
        dim: usize,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Cast { lhs, rhs } => {
                write!(
                    f,
                    "operands could not be broadcast together with shapes {}, {}",
                    lhs, rhs
                )
            }
            Self::Broadcast { dims1, dims2 } => {
                write!(
                    f,
                    "operands could not be broadcast together with shapes {:?}, {:?}",
                    dims1, dims2
                )
            }
            Self::Reshape { initial, target } => {
                write!(
                    f,
                    "cannot reshape: shapes have different volumes (a: {:?} -> {}, b: {:?} -> {})",
                    initial,
                    initial.volume(),
                    target,
                    target.volume()
                )
            }
            Self::ShapeZeroDims => {
                write!(f, "shape cannot be constructed with 0 dims")
            }
            Self::ShapeZeroLenDim { dims } => {
                write!(f, "shape cannot have a dim of width 0, received {:?}", dims)
            }
            Self::ShapeDataMisalignment {
                shape_volume,
                data_len,
            } => {
                write!(
                    f,
                    "shape volume is {}, but {} elements were provided",
                    shape_volume, data_len
                )
            }
            Self::ShapeDataMisalignment1 { shape, data_len } => {
                write!(
                    f,
                    "array expected {} elements, but shape has volume {}",
                    data_len,
                    shape.volume()
                )
            }
            Self::FromIdxFile { filename } => {
                write!(f, "couldn't create array from file: {}", filename)
            }
            Self::DerankIndexOutOfBounds { len, index } => {
                write!(f, "cannot derank at index {} when len is {}", index, len)
            }
            Self::ReadNDim { ndims } => {
                write!(f, "cannot read an individual value from an array with more than 1 dim, has {} dims", ndims)
            }
            Self::Derank1D => {
                write!(
                    f,
                    "cannot slice down to a smaller dimension from a 1D array"
                )
            }
            Self::SliceZeroWidth { index } => {
                write!(
                    f,
                    "slice cannot have 0 size, start: {i}, stop: {i}",
                    i = index
                )
            }
            Self::SliceStopBeforeStart { start, stop } => {
                write!(
                    f,
                    "slice start, {}, cannot be greater than stop, {}",
                    start, stop
                )
            }
            Self::SliceStopPastEnd {
                stop: slice_width,
                dim: dim_width,
            } => {
                write!(
                    f,
                    "cannot slice array of len {} with a slice of width {}",
                    dim_width, slice_width
                )
            }
        }
    }
}
