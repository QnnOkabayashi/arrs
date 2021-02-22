use crate::array::Shape;
use std::{
    fmt::{self, Display},
    result::Result,
};

pub type ArrResult<T> = Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Cast { lhs: Shape, rhs: Shape },
    Reshape { initial: Shape, target: Shape },
    ShapeDataMisalignment { shape: Shape, data_len: usize },
    FromIdxFile { filename: &'static str },
    DerankInvalidIndex { len: usize, index: usize },
    ReadNDim { ndims: usize },
    Derank1D,
    SliceZeroWidth { index: usize },
    SliceStopBeforeStart { start: usize, stop: usize },
    SliceStopPastEnd { stop: usize, dim: usize },
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
            Self::ShapeDataMisalignment { shape, data_len } => {
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
            Self::DerankInvalidIndex { len, index } => {
                write!(f, "can't derank at index {} when len is {}", index, len)
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
