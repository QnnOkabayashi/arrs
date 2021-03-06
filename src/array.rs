mod error;
#[cfg(not(no_std))]
mod idx;
mod shape;
#[macro_use]
mod macros;
use core::fmt::Debug;
use core::iter::{repeat, Sum};
use core::ops::{Add, Div, Mul, Sub};
pub use error::{ArrResult, Error};
use shape::BroadcastInstruction;
pub use shape::Shape;

// helper function for compile time use
pub const fn max_const(v1: usize, v2: usize) -> usize {
    if v1 > v2 {
        v1
    } else {
        v2
    }
}

pub const fn min_const(v1: usize, v2: usize) -> usize {
    if v1 < v2 {
        v1
    } else {
        v2
    }
}

/// A base for owning `Array` data
pub struct ArrayBase<T: ArrayType, const NDIMS: usize> {
    shape: Shape<NDIMS>,
    data: Vec<T>,
}

impl<T: ArrayType, const NDIMS: usize> ArrayBase<T, NDIMS> {
    pub fn new(shape: Shape<NDIMS>, data: Vec<T>) -> ArrResult<Self> {
        let volume = shape.volume();

        if volume != data.len() {
            return Err(Error::ShapeDataMisalignment {
                volume,
                data_len: data.len(),
            });
        }

        Ok(Self { shape, data })
    }
}

/// A view into an `ArrayBase` object
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Array<'base, T: ArrayType, const NDIMS: usize> {
    shape: Shape<NDIMS>,
    data: &'base [T],
}

impl<'base, T: ArrayType, const NDIMS: usize> Array<'base, T, NDIMS> {
    /// Combine `Array`s of different sizes using array broadcasting
    pub fn broadcast_combine<const NDIMS2: usize, F>(
        &self,
        other: &Array<T, NDIMS2>,
        combinator: F,
    ) -> ArrResult<ArrayBase<T, { max_const(NDIMS, NDIMS2) }>>
    where
        F: Fn(T, T) -> T,
    {
        match self.shape.broadcast(&other.shape) {
            Ok((shape, broadcast_instructions)) => {
                use BroadcastInstruction::*;
                let mut data = Vec::with_capacity(shape.volume());

                fn recurse<'base1, T, F>(
                    a: &[T],
                    b: &[T],
                    instructions: &[BroadcastInstruction],
                    out: &mut Vec<T>,
                    f: &F,
                ) where
                    T: ArrayType,
                    F: Fn(T, T) -> T,
                {
                    let (instruction, instructions2) = instructions.split_last().unwrap();

                    match *instruction {
                        PushLinear => {
                            out.extend(a.iter().zip(b.iter()).map(|(&a_n, &b_n)| f(a_n, b_n)));
                        }
                        PushStretchA => {
                            out.extend(b.iter().map(|&b_n| f(a[0], b_n)));
                        }
                        PushStretchB => {
                            out.extend(a.iter().map(|&a_n| f(a_n, b[0])));
                        }
                        RecurseLinear { stride_a, stride_b } => {
                            for (a2, b2) in a.chunks_exact(stride_a).zip(b.chunks_exact(stride_b)) {
                                recurse(a2, b2, instructions2, out, f);
                            }
                        }
                        RecurseStretchA { stride_b } => {
                            for b2 in b.chunks_exact(stride_b) {
                                recurse(a, b2, instructions2, out, f);
                            }
                        }
                        RecurseStretchB { stride_a } => {
                            for a2 in a.chunks_exact(stride_a) {
                                recurse(a2, b, instructions2, out, f);
                            }
                        }
                    }
                }

                recurse(
                    self.data,
                    other.data,
                    &broadcast_instructions,
                    &mut data,
                    &combinator,
                );

                Ok(ArrayBase { shape, data })
            }
            Err(e) => Err(e),
        }
    }

    /// Convert the data type
    pub fn as_type<R: ArrayType + From<T>>(&self) -> ArrayBase<R, NDIMS> {
        ArrayBase {
            shape: self.shape,
            data: self.data.iter().map(|x| R::from(*x)).collect(),
        }
    }

    /// Generate fresh `Array` from an `ArrayBase`
    pub fn from_base(base: &'base ArrayBase<T, NDIMS>) -> Self {
        Self {
            shape: base.shape,
            data: &base.data[..],
        }
    }

    /// Generate fresh `ArrayBase` from this `Array`
    pub fn into_base(&self) -> ArrayBase<T, NDIMS> {
        ArrayBase {
            shape: self.shape,
            data: self.data.to_vec(),
        }
    }

    pub fn shape(&self) -> &Shape<NDIMS> {
        &self.shape
    }

    /// Matrix multiplication for 2x2, 2x1, 1x2, and 1x1 `Array`s
    pub fn matmul<const NDIMS2: usize>(
        &self,
        other: &Array<T, NDIMS2>,
    ) -> ArrResult<ArrayBase<T, { min_const(NDIMS, NDIMS2) }>>
    where
        [(); 2 - NDIMS]: , // at most 2
        [(); NDIMS - 1]: , // at least 1
        [(); 2 - NDIMS2]: ,
        [(); NDIMS2 - 1]: ,
    {
        // can probably figure out which one at compile time
        match (NDIMS, NDIMS2) {
            (2, 2) => {
                // matrix matrix
                todo!()
            }
            (2, 1) => {
                // matrix vector
                let (rows_a, cols_a) = (self.shape.dims()[0], self.shape.dims()[1]);
                let len_b = other.shape.dims()[0];
                if rows_a != len_b {
                    return Err(Error::MatMul {
                        rows_a,
                        cols_b: len_b,
                    });
                }

                Ok(ArrayBase {
                    shape: Shape::new([cols_a; min_const(NDIMS, NDIMS2)]),
                    data: (self.data.chunks_exact(cols_a).zip(repeat(other.data)).map(
                        |(a_row, b_col)| {
                            a_row
                                .iter()
                                .zip(b_col.iter())
                                .map(|(&a_val, &b_val)| a_val * b_val)
                                .sum()
                        },
                    ))
                    .collect(),
                })
            }
            (1, 2) => {
                // vector matrix
                todo!()
            }
            (1, 1) => {
                // vector vector
                let len_a = self.shape.dims()[0];
                let len_b = other.shape.dims()[0];
                if len_a != len_b {
                    return Err(Error::MatMul {
                        rows_a: len_a,
                        cols_b: len_b,
                    });
                }

                Ok(ArrayBase {
                    shape: Shape::new([1; min_const(NDIMS, NDIMS2)]), // always 1 length
                    data: vec![self
                        .data
                        .iter()
                        .zip(other.data.iter())
                        .map(|(&a, &b)| a * b)
                        .sum()],
                })
            }
            _ => unreachable!(),
        }
    }
}

pub trait ArrayType:
    Copy
    + PartialEq
    + Debug
    + Sum
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
}

impl_arraytype! { u8, i8, i16, i32, f32, f64 }
