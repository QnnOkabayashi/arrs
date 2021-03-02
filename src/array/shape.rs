use crate::array::{ArrResult, Error, PartialView};
use core::{cmp, iter::once};

#[derive(Debug)]
pub struct ShapeBase {
    dims: Vec<usize>,
    volumes: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape<'a> {
    len: usize,
    volume: usize,
    sub_dims: &'a [usize],
    sub_volumes: &'a [usize],
}

impl ShapeBase {
    pub(self) fn new(dims: Vec<usize>) -> Self {
        let volumes = once(1)
            .chain(dims.iter().scan(1, |volume, &dim| {
                *volume *= dim;
                Some(*volume)
            }))
            .collect();

        Self { dims, volumes }
    }

    pub fn new_checked(dims: Vec<usize>) -> ArrResult<Self> {
        if dims.len() == 0 {
            return Err(Error::ShapeZeroDims);
        } else if dims.iter().any(|&dim| dim == 0) {
            return Err(Error::ShapeZeroLenDim { dims });
        }

        Ok(Self::new(dims))
    }

    pub(super) fn total_volume(&self) -> usize {
        *self.volumes.last().unwrap()
    }
}

impl<'a> Shape<'a> {
    pub(super) fn ndims(&self) -> usize {
        self.sub_dims.len() + 1
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub fn volume(&self) -> usize {
        self.volume
    }

    fn dims_iter(&self) -> impl DoubleEndedIterator<Item = &usize> {
        self.sub_dims.iter().chain(once(&self.len))
    }

    pub fn to_vec(&self) -> Vec<usize> {
        self.dims_iter().copied().collect()
    }

    pub(super) fn stride(&self) -> usize {
        // SAFETY: sub_volumes always starts with an extra 1
        *self.sub_volumes.last().unwrap()
    }

    pub fn derank(&self) -> Self {
        let (&len, sub_dims) = self.sub_dims.split_last().unwrap();
        let (&volume, sub_volumes) = self.sub_volumes.split_last().unwrap();

        Self {
            len,
            volume,
            sub_dims,
            sub_volumes,
        }
    }

    fn slice(&self, len: usize) -> Self {
        let volume = len * self.sub_volumes.last().unwrap();

        Self {
            len,
            volume,
            sub_dims: self.sub_dims,
            sub_volumes: self.sub_volumes,
        }
    }

    pub(super) fn derank_checked(&self, index: usize) -> ArrResult<Shape> {
        if self.sub_dims.len() == 0 {
            return Err(Error::Derank1D);
        } else if index >= self.len {
            return Err(Error::DerankIndexOutOfBounds {
                len: self.len,
                index,
            });
        }

        Ok(self.derank())
    }

    pub(super) fn slice_checked(&self, start: usize, stop: usize) -> ArrResult<Shape> {
        if start == stop {
            return Err(Error::SliceZeroWidth { index: start });
        } else if start > stop {
            return Err(Error::SliceStopBeforeStart { start, stop });
        } else if stop > self.len {
            return Err(Error::SliceStopPastEnd {
                stop,
                dim: self.len,
            });
        }

        Ok(self.slice(stop - start))
    }

    pub fn broadcast(&self, other: &Self) -> ArrResult<(ShapeBase, Vec<BroadcastInstruction>)> {
        use BroadcastInstruction::*;
        let result_ndims = cmp::max(self.ndims(), other.ndims());
        let mut dims = Vec::with_capacity(result_ndims);
        let mut broadcast_instructions = Vec::with_capacity(result_ndims);

        let mut iter1 = self.dims_iter();
        let mut iter2 = other.dims_iter();

        loop {
            match (iter1.next(), iter2.next()) {
                (None, None) => break,
                (Some(&a), Some(&b)) if a == b => {
                    broadcast_instructions.push(RecurseLinear);
                    dims.push(a);
                }
                (Some(&a), Some(1)) => {
                    broadcast_instructions.push(RecurseStretchB);
                    dims.push(a);
                }
                (Some(1), Some(&b)) => {
                    broadcast_instructions.push(RecurseStretchA);
                    dims.push(b);
                }
                (Some(&a), None) => {
                    broadcast_instructions.push(RecursePadB);
                    dims.push(a);
                }
                (None, Some(&b)) => {
                    broadcast_instructions.push(RecursePadA);
                    dims.push(b);
                }
                _ => {
                    return Err(Error::Broadcast {
                        dims1: self.to_vec(),
                        dims2: other.to_vec(),
                    })
                }
            }
        }

        broadcast_instructions[0] = match broadcast_instructions[0] {
            RecurseLinear => PushLinear,
            RecurseStretchA => PushStretchA,
            RecurseStretchB => PushStretchB,
            _ => unreachable!(),
        };

        Ok((ShapeBase::new(dims), broadcast_instructions))
    }
}

impl<'base> PartialView<'base> for Shape<'base> {
    type Base = ShapeBase;

    fn from_base(base: &'base Self::Base) -> Self {
        let (&len, sub_dims) = base.dims.split_last().unwrap();
        let (&volume, sub_volumes) = base.volumes.split_last().unwrap();

        Self {
            len,
            volume,
            sub_dims,
            sub_volumes,
        }
    }

    fn into_base(&self) -> Self::Base {
        // recalculating volume is O(n)
        // copying with a chain iter is O(n)
        // may as well recalculate
        ShapeBase::new(self.dims_iter().copied().collect())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BroadcastInstruction {
    PushLinear,
    PushStretchA,
    PushStretchB,
    RecurseLinear,
    RecurseStretchA,
    RecurseStretchB,
    RecursePadA,
    RecursePadB,
}
