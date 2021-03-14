use super::{
    min_const, ArrResult, ArrType, ArrayBase, ArrayData, ArrayIndexer, Error, MultiDimensional,
};

use core::convert::TryInto;
use core::iter::repeat;
use core::ops::Range;
use core::slice::{ChunksExact, Iter};

/// A view into an `ArrayBase` object
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Array<'base, T: ArrType, const NDIMS: usize> {
    dims: [usize; NDIMS], // innermost first, outermost last
    data: &'base [T],
}

impl<'base, T: ArrType, const NDIMS: usize> Array<'base, T, NDIMS> {


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

                let dims = [cols_a; min_const(NDIMS, NDIMS2)];
                let data = self
                    .data()
                    .chunks_exact(cols_a)
                    .zip(repeat(other.data()))
                    .map(|(a_row, b_col)| {
                        a_row
                            .iter()
                            .zip(b_col.values_iter())
                            .map(|(&a_val, &b_val)| a_val * b_val)
                            .sum()
                    })
                    .collect();

                Ok(ArrayBase::from_raw_parts(dims, data))
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

                let dims = [1; min_const(NDIMS, NDIMS2)];
                let data = vec![self
                    .data
                    .iter()
                    .zip(other.data.iter())
                    .map(|(&a, &b)| a * b)
                    .sum()];

                Ok(ArrayBase::from_raw_parts(dims, data))
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
}

impl<'base, T: 'base + ArrType, const NDIMS: usize> MultiDimensional<'base, T, NDIMS>
    for Array<'base, T, NDIMS>
{
    type Data = &'base [T];

    type Indexer = Range<usize>;

    fn dims(&self) -> &[usize] {
        &self.dims
    }

    fn data(&self) -> Self::Data {
        self.data
    }

    fn as_type<R: ArrType + From<T>>(&self) -> ArrayBase<R, NDIMS> {
        let data = self.data.iter().map(|x| R::from(*x)).collect();

        ArrayBase::from_raw_parts(self.dims, data)
    }

    fn from_base(base: &'base ArrayBase<T, NDIMS>) -> Self {
        Self {
            dims: base.dims(),
            data: base.data(),
        }
    }

    fn into_base(&self) -> ArrayBase<T, NDIMS> {
        ArrayBase::from_raw_parts(self.dims, self.data.to_vec())
    }

    fn slice(&self, indexer: Self::Indexer) -> ArrResult<Self> {
        let (&len, dims_slice) = self.dims.split_last().unwrap();

        let mut dims = self.dims.clone();
        *dims.last_mut().unwrap() = indexer.size(len)?;

        let stride = dims_slice.iter().product::<usize>();
        let data = &self.data[stride * indexer.start()..stride * indexer.end()];

        Ok(Array { dims, data })
    }
}

impl<'base, T: ArrType> ArrayData<'base, T> for &'base [T] {
    type ValueIterator = Iter<'base, T>;

    type ChunksIterator = ChunksExact<'base, T>;

    fn values_first(&self) -> T {
        self[0]
    }

    fn values_iter(&self) -> Self::ValueIterator {
        self.iter()
    }

    fn derank_first(&self) -> Self {
        self
    }

    fn derank_iter(&self, stride: usize) -> Self::ChunksIterator {
        self.chunks_exact(stride)
    }
}

impl ArrayIndexer for Range<usize> {
    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

#[test]
fn broadcast_test() -> ArrResult<()> {
    use crate::arrs;
    arrs!(let arr1 = [10]);
    arrs!(let arr2 = [0,1,2,3,4]);

    arrs!(let expected = [0,10,20,30,40]);
    arrs!(let actual = mul(&arr1, &arr2));

    Ok(assert_eq!(expected, actual))
}
