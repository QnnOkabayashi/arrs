mod dense;
// mod sparse;
pub use dense::Array as DenseArray;
// pub use sparse::Array as SparseArray;
use crate::base::ArrayBase;
use crate::error::{ArrResult, Error};
use crate::types::ArrType;

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

pub trait MultiDimensional<'base, T: 'base + ArrType, const NDIMS: usize>: Sized {
    type Data: ArrayData<'base, T>;

    type Indexer: ArrayIndexer;

    // do I need to have it take self?
    fn ndims() -> usize {
        NDIMS
    }

    fn dims(&self) -> &[usize];

    fn data(&self) -> Self::Data;

    fn broadcast_combine<const NDIMS2: usize, F, A>(
        &self,
        other: &A,
        combinator: F,
    ) -> ArrResult<ArrayBase<T, { max_const(NDIMS, NDIMS2) }>>
    where
        F: Fn(T, T) -> T,
        A: MultiDimensional<'base, T, NDIMS2>,
    {
        #[derive(Clone, Copy)]
        pub enum Instruction {
            PushLinear,
            PushStretchA,
            PushStretchB,
            RecurseLinear { stride_a: usize, stride_b: usize },
            RecurseStretchA { stride_b: usize },
            RecurseStretchB { stride_a: usize },
            RecursePadA { stride_b: usize },
            RecursePadB { stride_a: usize },
        }
        use Instruction::*;

        let (dims, instrs) = {
            let mut dims = [0; max_const(NDIMS, NDIMS2)];
            let mut instrs = [PushLinear; max_const(NDIMS, NDIMS2)];

            let (mut iter_a, mut iter_b) = (self.dims().iter(), other.dims().iter());
            let (mut stride_a, mut stride_b) = (1, 1);

            for (dim, instruction) in dims.iter_mut().zip(instrs.iter_mut()) {
                let (next_a, next_b) = (iter_a.next(), iter_b.next());

                let (d, i) = match (next_a, next_b) {
                    (Some(a), Some(b)) if a == b => (a, RecurseLinear { stride_a, stride_b }),
                    (Some(a), Some(1)) => (a, RecurseStretchB { stride_a }),
                    (Some(1), Some(b)) => (b, RecurseStretchA { stride_b }),
                    (Some(a), None) => (a, RecursePadB { stride_a }),
                    (None, Some(b)) => (b, RecursePadA { stride_b }),
                    (None, None) => unreachable!(),
                    _ => {
                        return Err(Error::Broadcast {
                            dims1: self.dims().to_vec(),
                            dims2: other.dims().to_vec(),
                        })
                    }
                };

                stride_a *= next_a.unwrap_or(&1);
                stride_b *= next_b.unwrap_or(&1);

                *dim = *d;
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

        // loose types for future modularity
        // when the compiler doesn't panic :(
        fn recurse<'base1, 'base2, T1, T2, R, F, D1, D2>(
            a: D1,
            b: D2,
            instrs: &[Instruction],
            out: &mut Vec<R>,
            f: &F,
        ) where
            D1: ArrayData<'base1, T1>,
            D2: ArrayData<'base2, T2>,
            T1: 'base1 + ArrType,
            T2: 'base2 + ArrType,
            R: ArrType,
            F: Fn(T1, T2) -> R,
        {
            let (instr, instrs) = instrs.split_last().unwrap();

            match *instr {
                PushLinear => {
                    out.extend(
                        a.values_iter()
                            .zip(b.values_iter())
                            .map(|(&a_n, &b_n)| f(a_n, b_n)),
                    );
                }
                PushStretchA => {
                    out.extend(b.values_iter().map(|&b_n| f(a.values_first(), b_n)));
                }
                PushStretchB => {
                    out.extend(a.values_iter().map(|&a_n| f(a_n, b.values_first())));
                }
                RecurseLinear { stride_a, stride_b } => {
                    for (a2, b2) in a.derank_iter(stride_a).zip(b.derank_iter(stride_b)) {
                        recurse(a2, b2, instrs, out, f);
                    }
                }
                RecurseStretchA { stride_b } => {
                    for b_deranked in b.derank_iter(stride_b) {
                        recurse(a.derank_first(), b_deranked, instrs, out, f);
                    }
                }
                RecurseStretchB { stride_a } => {
                    for a_deranked in a.derank_iter(stride_a) {
                        recurse(a_deranked, b.derank_first(), instrs, out, f);
                    }
                }
                RecursePadA { stride_b } => {
                    for b_deranked in b.derank_iter(stride_b) {
                        recurse(a, b_deranked, instrs, out, f);
                    }
                }
                RecursePadB { stride_a } => {
                    for a_deranked in a.derank_iter(stride_a) {
                        recurse(a_deranked, b, instrs, out, f);
                    }
                }
            }
        }

        let mut data = Vec::with_capacity(dims.iter().product());

        // Wish we could make `recurse` return some type of iterator
        // that could just be collected to make this more idiomatic.
        // Unfortunately, this would require `Box`ing the iterators
        // which isn't idiomatic at all.
        recurse(self.data(), other.data(), &instrs, &mut data, &combinator);

        Ok(ArrayBase::from_raw_parts(dims, data))
    }

    fn as_type<R: ArrType + From<T>>(&self) -> ArrayBase<R, NDIMS>;

    fn from_base(base: &'base ArrayBase<T, NDIMS>) -> Self;

    fn into_base(&self) -> ArrayBase<T, NDIMS>;

    fn slice(&self, indexer: Self::Indexer) -> ArrResult<Self>;
}

pub trait ArrayData<'a, T: 'a + ArrType>: Copy {
    type ValueIterator: Iterator<Item = &'a T>;

    type ChunksIterator: Iterator<Item = Self>;

    fn values_first(&self) -> T;

    fn values_iter(&self) -> Self::ValueIterator;

    fn derank_first(&self) -> Self;

    fn derank_iter(&self, stride: usize) -> Self::ChunksIterator;
}

pub trait ArrayIndexer {
    fn start(&self) -> usize;

    fn end(&self) -> usize;

    fn step(&self) -> isize {
        1
    }

    fn size(&self, len: usize) -> ArrResult<usize> {
        match (self.start(), self.end(), self.step()) {
            (_, _, step) if step == 0 => Err(Error::SliceZeroStep),
            (start, end, _) if start == end => Err(Error::SliceZeroWidth { index: start }),
            (start, end, step) if (start < end) != (step > 0) => {
                Err(Error::SliceNonConverging { start, end, step })
            }
            (start, _, _) if start > len => Err(Error::SliceStartOutOfBounds { start, len }),
            (_, end, _) if end > len => Err(Error::SliceEndOutOfBounds { end, len }),
            (start, end, step) => {
                let distance = (start as isize - end as isize).abs();
                let size = ((distance - 1) / step.abs() + 1) as usize;

                if size > len {
                    Err(Error::SliceTooLarge { size, len })
                } else {
                    Ok(size)
                }
            }
        }
    }
}
