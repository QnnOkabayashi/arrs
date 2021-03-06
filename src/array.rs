mod error;
#[cfg(not(no_std))]
mod idx;
// mod shape;
#[macro_use]
mod macros;
use core::fmt::Debug;
use core::iter::{repeat, Sum};
use core::ops::{Add, Div, Mul, Sub};
pub use error::{ArrResult, Error};
use std::convert::TryInto;

// helper functions for compile time use
pub const fn max_const(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

pub const fn min_const(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}

/// A base for owning `Array` data
pub struct ArrayBase<T: ArrType, const NDIMS: usize> {
    dims: [usize; NDIMS],
    data: Vec<T>,
}

impl<T: ArrType, const NDIMS: usize> ArrayBase<T, NDIMS> {
    pub fn new(dims: [usize; NDIMS], data: Vec<T>) -> ArrResult<Self> {
        let (volume, len) = (dims.iter().product(), data.len());
        if NDIMS == 0 {
            Err(Error::ShapeZeroDims)
        } else if volume != len {
            Err(Error::ShapeDataMisalignment { volume, len })
        } else {
            Ok(Self { dims, data })
        }
    }
}

/// A view into an `ArrayBase` object
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Array<'base, T: ArrType, const NDIMS: usize> {
    dims: [usize; NDIMS], // innermost first, outermost last
    data: &'base [T],
}

impl<'base, T: ArrType, const NDIMS: usize> Array<'base, T, NDIMS> {
    /// Get the number of dimensions
    pub fn ndims(&self) -> usize {
        NDIMS
    }

    /// Combine `Array`s of different sizes using array broadcasting
    pub fn broadcast_combine<const NDIMS2: usize, F: Fn(T, T) -> T>(
        &self,
        other: &Array<T, NDIMS2>,
        combinator: F,
    ) -> ArrResult<ArrayBase<T, { max_const(NDIMS, NDIMS2) }>> {
        #[derive(Clone, Copy)]
        pub enum Instruction {
            PushLinear,
            PushStretchA,
            PushStretchB,
            RecurseLinear { stride_a: usize, stride_b: usize },
            RecurseStretchA { stride_b: usize },
            RecurseStretchB { stride_a: usize },
        }
        use Instruction::*;

        let (dims, instrs) = {
            let mut dims = [0; max_const(NDIMS, NDIMS2)];
            let mut instrs = [PushLinear; max_const(NDIMS, NDIMS2)];

            let (mut iter_a, mut iter_b) = (self.dims.iter(), other.dims.iter());
            let (mut stride_a, mut stride_b) = (1, 1);

            for (dim, instruction) in dims.iter_mut().zip(instrs.iter_mut()) {
                let (next_a, next_b) = (iter_a.next(), iter_b.next());

                let (d, i) = match (next_a, next_b) {
                    (Some(&a), Some(&b)) if a == b => (a, RecurseLinear { stride_a, stride_b }),
                    (Some(&a), Some(1)) | (Some(&a), None) => (a, RecurseStretchB { stride_a }),
                    (Some(1), Some(&b)) | (None, Some(&b)) => (b, RecurseStretchA { stride_b }),
                    (None, None) => unreachable!(),
                    _ => {
                        return Err(Error::Broadcast {
                            dims1: self.dims.to_vec(),
                            dims2: other.dims.to_vec(),
                        })
                    }
                };

                stride_a *= next_a.unwrap_or(&1);
                stride_b *= next_b.unwrap_or(&1);

                *dim = d;
                *instruction = i;
            }

            instrs[0] = match instrs[0] {
                RecurseLinear { .. } => PushLinear,
                RecurseStretchA { .. } => PushStretchA,
                RecurseStretchB { .. } => PushStretchB,
                _ => unreachable!(),
            };

            (dims, instrs)
        };

        // take some data type that has some equivalent to iter() and chunks_exact()
        // where iter returns values, and chunks_exact returns same type as current
        fn recurse<T, F>(a: &[T], b: &[T], instrs: &[Instruction], out: &mut Vec<T>, f: &F)
        where
            T: ArrType,
            F: Fn(T, T) -> T,
        {
            let (instr, instrs) = instrs.split_last().unwrap();

            match *instr {
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
                        recurse(a2, b2, instrs, out, f);
                    }
                }
                RecurseStretchA { stride_b } => {
                    for b2 in b.chunks_exact(stride_b) {
                        recurse(a, b2, instrs, out, f);
                    }
                }
                RecurseStretchB { stride_a } => {
                    for a2 in a.chunks_exact(stride_a) {
                        recurse(a2, b, instrs, out, f);
                    }
                }
            }
        }

        let mut data = Vec::with_capacity(dims.iter().product());

        recurse(self.data, other.data, &instrs, &mut data, &combinator);

        Ok(ArrayBase { dims, data })
    }

    /// Convert the data type
    pub fn as_type<R: ArrType + From<T>>(&self) -> ArrayBase<R, NDIMS> {
        ArrayBase {
            dims: self.dims,
            data: self.data.iter().map(|x| R::from(*x)).collect(),
        }
    }

    /// Generate fresh `Array` from an `ArrayBase`
    pub fn from_base(base: &'base ArrayBase<T, NDIMS>) -> Self {
        Self {
            dims: base.dims,
            data: &base.data[..],
        }
    }

    /// Generate fresh `ArrayBase` from this `Array`
    pub fn into_base(&self) -> ArrayBase<T, NDIMS> {
        ArrayBase {
            dims: self.dims,
            data: self.data.to_vec(),
        }
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
                let (rows_a, cols_a) = (self.dims[0], self.dims[1]);
                let len_b = other.dims[0];
                if rows_a != len_b {
                    return Err(Error::MatMul {
                        rows_a,
                        cols_b: len_b,
                    });
                }

                Ok(ArrayBase {
                    dims: [cols_a; min_const(NDIMS, NDIMS2)],
                    data: self
                        .data
                        .chunks_exact(cols_a)
                        .zip(repeat(other.data))
                        .map(|(a_row, b_col)| {
                            a_row
                                .iter()
                                .zip(b_col.iter())
                                .map(|(&a_val, &b_val)| a_val * b_val)
                                .sum()
                        })
                        .collect(),
                })
            }
            (1, 2) => {
                // vector matrix
                todo!()
            }
            (1, 1) => {
                // vector vector (dot product)
                let len_a = self.dims[0];
                let len_b = other.dims[0];
                if len_a != len_b {
                    return Err(Error::MatMul {
                        rows_a: len_a,
                        cols_b: len_b,
                    });
                }

                Ok(ArrayBase {
                    dims: [1; min_const(NDIMS, NDIMS2)], // always 1 length
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

    pub fn derank(&self, index: usize) -> ArrResult<Array<T, { NDIMS - 1 }>>
    where
        [(); NDIMS - 2]: ,
    {
        let (&len, dims_slice) = self.dims.split_last().unwrap();
        if index >= len {
            return Err(Error::DerankIndexOutOfBounds { len, index });
        }

        let stride = dims_slice.iter().product::<usize>();

        Ok(Array {
            dims: dims_slice.try_into().unwrap(),
            data: &self.data[stride * index..stride * (index + 1)],
        })
    }

    pub fn slice(&self, start: usize, stop: usize) -> ArrResult<Self> {
        let (&len, dims_slice) = self.dims.split_last().unwrap();
        if stop < start {
            return Err(Error::SliceStopBeforeStart { start, stop });
        } else if stop == start {
            return Err(Error::SliceZeroWidth { index: start });
        } else if stop > len {
            return Err(Error::SliceStopPastEnd { stop, len });
        }

        let stride = dims_slice.iter().product::<usize>();

        let mut dims = self.dims.clone();
        *dims.last_mut().unwrap() = stop - start;

        Ok(Array {
            dims,
            data: &self.data[stride * start..stride * stop],
        })
    }
}

pub trait ArrType:
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

impl ArrType for u8 {}
impl ArrType for i8 {}
impl ArrType for i16 {}
impl ArrType for i32 {}
impl ArrType for f32 {}
impl ArrType for f64 {}
