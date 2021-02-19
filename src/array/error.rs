use crate::array::Shape;
use std::{
    fmt::{self, Display},
    result::Result,
};

pub type ArrResult<T> = Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Cast(Shape, Shape),
    ShapeDataMisalignment(Shape, usize),
    Reshape(Shape, Shape),
    FromIdxFile(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Cast(s1, s2) => {
                write!(
                    f,
                    "operands could not be broadcast together with shapes {}, {}",
                    s1, s2
                )
            }
            Self::Reshape(s1, s2) => {
                write!(
                    f,
                    "cannot reshape: shapes have different volumes (a: {:?} -> {}, b: {:?} -> {})",
                    s1,
                    s1.volume(),
                    s2,
                    s2.volume()
                )
            }
            Self::ShapeDataMisalignment(shape, len) => {
                write!(
                    f,
                    "array expected {} elements, but shape has volume {}",
                    len,
                    shape.volume()
                )
            }
            Self::FromIdxFile(filename) => {
                write!(f, "couldn't create array from file: {}", filename)
            }
        }
    }
}
