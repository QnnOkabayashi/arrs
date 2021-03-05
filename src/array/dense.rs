use crate::array::{ArrResult, ArrayBase, Error, MultiDimensional, PartialView, Shape, TypeAware};
use core::slice::{ChunksExact, Iter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Array<'base, T: TypeAware> {
    shape: Shape<'base>,
    data: &'base [T],
}

impl<'a, T: TypeAware> Array<'a, T> {
    pub fn len(&self) -> usize {
        self.data.len()
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

impl<'base, T: TypeAware> MultiDimensional<'base, T> for Array<'base, T> {
    type SubviewIterator = DenseIterator<'base, T>;

    fn ndims(&self) -> usize {
        self.shape.ndims()
    }

    fn shape(&self) -> &Shape {
        &self.shape
    }

    fn one_value(&self) -> T {
        self.data[0]
    }

    fn iter_values(&self) -> Iter<T> {
        self.data.iter()
    }

    fn one_subview(&self) -> Self {
        let shape = self.shape.derank();
        let data = &self.data[..self.shape.stride()];

        Self { shape, data }
    }

    fn iter_subviews(&self) -> Self::SubviewIterator {
        let shape = self.shape.derank();
        let chunks = self.data.chunks_exact(shape.volume());

        Self::SubviewIterator { shape, chunks }
    }
}

pub struct DenseIterator<'base, T: TypeAware> {
    shape: Shape<'base>,
    chunks: ChunksExact<'base, T>,
}

impl<'base, T: TypeAware> Iterator for DenseIterator<'base, T> {
    type Item = Array<'base, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next().map(|data| Array {
            shape: self.shape,
            data,
        })
    }
}
