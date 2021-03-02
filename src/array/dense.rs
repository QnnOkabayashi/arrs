use crate::array::{ArrResult, ArrayBase, Broadcastable, Error, PartialView, Shape, TypeAware};
use core::ops::{Add, Div, Mul, Rem, Sub};
use core::slice::Iter;
use std::slice::ChunksExact;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// DenseArray is a PartialView of ArrayBase where elements
// are stored contiguously in memory. This means fast
// but restrictive array indexing
pub struct Array<'base, T: TypeAware> {
    shape: Shape<'base>,
    data: &'base [T],
}

impl<'a, T: TypeAware> Array<'a, T> {
    pub fn ndims(&self) -> usize {
        self.shape.ndims()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    fn at(&self, index: usize) -> T {
        // todo: implement Index trait later
        // keep this for clarity rn
        self.data[index]
    }

    // instead of having an `at` method, wouldn't it make
    // more sense to just have a method to iterate over
    // the elements? more compiler optimizations probably

    pub fn at_checked(&self, index: usize) -> ArrResult<T> {
        if self.ndims() > 1 {
            return Err(Error::ReadNDim {
                ndims: self.ndims(),
            });
        } else if index >= self.len() {
            return Err(Error::DerankIndexOutOfBounds {
                len: self.len(),
                index,
            });
        }

        Ok(self.at(index))
    }

    pub fn derank(&'a self, index: usize) -> ArrResult<Self> {
        // TODO: make iterate through deranked instead
        // of accessing them one by one
        let shape = self.shape.derank_checked(index)?;
        let step = self.shape.stride();
        let data = &self.data[index * step..(index + 1) * step];

        Ok(Self { shape, data })
    }

    pub fn slice(&'a self, start: usize, stop: usize) -> ArrResult<Self> {
        let shape = self.shape.slice_checked(start, stop)?;
        let step = self.shape.stride();
        let data = &self.data[step * start..step * stop];

        Ok(Self { shape, data })
    }
}

impl<'base, T: TypeAware> PartialView<'base> for Array<'base, T> {
    type Base = ArrayBase<T>;

    fn from_base(base: &'base Self::Base) -> Self {
        Self {
            shape: Shape::from_base(&base.shape_base),
            data: &base.data[..],
        }
    }

    fn into_base(&self) -> Self::Base {
        ArrayBase {
            shape_base: self.shape.into_base(),
            data: self.data.to_vec(),
        }
    }
}

impl<'base, T: TypeAware> Broadcastable<'base, T> for Array<'base, T> {
    type SubIterator = SubIterator<'base, T>;

    fn one_data(&self) -> T {
        self.data[0]
    }

    fn iter_flat_data(&self) -> Iter<T> {
        self.data.iter()
    }

    fn one_subarray(&self) -> Self {
        let shape = self.shape.derank();
        let data = &self.data[..self.shape.stride()];

        Self { shape, data }
    }

    fn iter_subarray(&self) -> Self::SubIterator {
        let shape = self.shape.derank();
        let chunks = self.data.chunks_exact(shape.volume());

        Self::SubIterator { shape, chunks }
    }

    fn shape(&self) -> &Shape {
        &self.shape
    }
}

macro_rules! impl_array_cmp {
    { $( $name:ident: $e:expr ),* } => {
        impl<'a, T> Array<'a, T>
        where
            T: TypeAware + PartialOrd
        {
            $(
                pub fn $name(&self, other: &Self) -> ArrResult<ArrayBase<bool>> {
                    self.broadcast_combine(other, $e)
                }
            )*
        }
    }
}

macro_rules! impl_array_op {
    { $( $op:ident($op_trait:path): $func:expr ),* } => {
        $(
            impl<'a, T> Array<'a, T>
            where
                T: TypeAware + $op_trait,
                <T as $op_trait>::Output: TypeAware,
            {
                pub fn $op(&self, other: &Self) -> ArrResult<ArrayBase<<T as $op_trait>::Output>> {
                    self.broadcast_combine(other, $func)
                }
            }
        )*
    }
}

impl_array_cmp! {
    v_eq: |a, b| a == b,
    v_ne: |a, b| a != b,
    v_lt: |a, b| a < b,
    v_le: |a, b| a <= b,
    v_gt: |a, b| a > b,
    v_ge: |a, b| a >= b
}

impl_array_op! {
    add(Add): |a, b| a + b,
    sub(Sub): |a, b| a - b,
    mul(Mul): |a, b| a * b,
    div(Div): |a, b| a / b,
    rem(Rem): |a, b| a % b
}

pub struct SubIterator<'base, T: TypeAware> {
    shape: Shape<'base>,
    chunks: ChunksExact<'base, T>,
}

impl<'base, T: TypeAware> Iterator for SubIterator<'base, T> {
    type Item = Array<'base, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next().map(|data| Array {
            shape: self.shape,
            data,
        })
    }
}
