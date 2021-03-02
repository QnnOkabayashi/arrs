use core::convert::From;
use std::{
    fmt::{self, Display},
    io::Error as IoError,
    result::Result,
};

pub type ArrResult<T> = Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Broadcast {
        dims1: Vec<usize>,
        dims2: Vec<usize>,
    },
    ShapeZeroDims,
    ShapeZeroLenDim {
        dims: Vec<usize>,
    },
    ShapeDataMisalignment {
        shape_volume: usize,
        data_len: usize,
    },
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
    IdxIO {
        // io::Error doesn't impl PartialEq, which is annoying
        message: String,
    },
    IdxReadUnaccepted,
    IdxWriteUnaccepted,
    MismatchDTypeIDs {
        dtype1: u8,
        dtype2: u8,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Broadcast { dims1, dims2 } => {
                write!(
                    f,
                    "operands could not be broadcast together with shapes {:?}, {:?}",
                    dims1, dims2
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
            Self::IdxIO { message } => f.write_str(&message),
            Self::IdxReadUnaccepted => {
                write!(f, "reader no longer providing bytes")
            }
            Self::IdxWriteUnaccepted => {
                write!(f, "writer no longer accepting bytes")
            }
            Self::MismatchDTypeIDs { dtype1, dtype2 } => {
                write!(f, "expected dtype ID: {}, found ID: {}", dtype1, dtype2)
            }
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::IdxIO {
            message: err.to_string(),
        }
    }
}
