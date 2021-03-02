use crate::array::{Arc, ArrResult, Error, PartialView};
use core::{
    cmp::max,
    fmt::{self, Display},
    iter::once,
};

#[derive(Debug)]
pub struct ShapeBase {
    dims: Vec<usize>,
    volumes: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub(self) fn from_parts(
        len: usize,
        volume: usize,
        sub_dims: &'a [usize],
        sub_volumes: &'a [usize],
    ) -> Self {
        Self {
            len,
            volume,
            sub_dims,
            sub_volumes,
        }
    }

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

    fn derank(&self) -> Self {
        let (&len, sub_dims) = self.sub_dims.split_last().unwrap();
        let (&volume, sub_volumes) = self.sub_volumes.split_last().unwrap();

        Self::from_parts(len, volume, sub_dims, sub_volumes)
    }

    fn slice(&self, len: usize) -> Self {
        let volume = len * self.sub_volumes.last().unwrap();

        Shape::from_parts(len, volume, self.sub_dims, self.sub_volumes)
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
        let result_ndims = max(self.ndims(), other.ndims());
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
            _ => panic!(
                "This pattern is unreachable because it would require a shape with zero dims"
            ),
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

// lowest dimensions are first
#[derive(Debug, Clone)]
pub struct Shape1 {
    ndims: usize,
    dim: usize,
    volume: usize,
    dims: Arc<Vec<usize>>,
    volumes: Arc<Vec<usize>>,
}

impl Shape1 {
    pub fn new(dims: Vec<usize>) -> Self {
        assert!(dims.len() > 0, "shapes can't have 0 dims");
        assert!(
            dims.iter().all(|&dim| dim > 0),
            "shapes can't have a dim of 0"
        );

        let ndims = dims.len();

        let dims = Arc::new(dims);

        let mut volumes_vec = Vec::with_capacity(1 + dims.len());

        volumes_vec.push(1);

        volumes_vec.extend(dims.iter().scan(1, |volume, &dim| {
            *volume *= dim;
            Some(*volume)
        }));

        let volumes = Arc::new(volumes_vec);

        // won't panic because dims and volumes both have at least 1 item
        let dim = *dims.last().unwrap();
        let volume = *volumes.last().unwrap();

        Self::from_parts(ndims, dim, volume, dims, volumes)
    }

    fn from_parts(
        ndims: usize,
        dim: usize,
        volume: usize,
        dims: Arc<Vec<usize>>,
        volumes: Arc<Vec<usize>>,
    ) -> Self {
        Self {
            ndims,
            dim,
            volume,
            dims,
            volumes,
        }
    }

    pub fn cast(&self, other: &Shape1) -> ArrResult<Shape1> {
        let mut dims = Vec::with_capacity(max(self.ndims(), other.ndims()) as usize);

        let (lhs, rhs) = (self.get_dims(), other.get_dims());
        let (mut lhs, mut rhs) = (lhs.iter(), rhs.iter());

        while let Some(pair) = match (lhs.next(), rhs.next()) {
            (None, None) => None,
            pair => Some(pair),
        } {
            dims.push(match pair {
                (Some(&a), Some(&b)) if a == b => a,
                (Some(&a), Some(1)) | (Some(&a), None) => a,
                (Some(1), Some(&b)) | (None, Some(&b)) => b,
                // (Some(&a), Some(1) | None) => a, // requires experimental feature 'or_patterns'
                // (Some(1) | None, Some(&b)) => b, // until then, leave commented
                _ => {
                    return Err(Error::Cast {
                        lhs: self.clone(),
                        rhs: other.clone(),
                    })
                }
            })
        }

        // Can volumes be predetermined here? idk
        Ok(Shape1::new(dims))
    }

    pub fn ndims(&self) -> usize {
        self.ndims
    }

    pub fn len(&self) -> usize {
        self.dim
    }

    pub fn dim_at(&self, index: usize) -> usize {
        assert!(
            index < self.ndims(),
            "index: {}, ndims: {}",
            index,
            self.ndims()
        );
        self.dims[index]
    }

    pub fn volume(&self) -> usize {
        self.volume
    }

    pub fn inside_volume(&self) -> usize {
        self.volumes[self.ndims() - 1]
    }

    pub fn get_dims(&self) -> Vec<usize> {
        self.dims
            .iter()
            .copied()
            .take(self.ndims() - 1)
            .chain(once(self.dim))
            .collect()
    }

    pub fn derank(&self, index: usize) -> ArrResult<Shape1> {
        if self.ndims() == 1 {
            return Err(Error::Derank1D);
        } else if index >= self.dim {
            return Err(Error::DerankIndexOutOfBounds {
                len: self.dim,
                index,
            });
        }

        let ndims = self.ndims - 1;
        let dim = self.dims[ndims - 1];
        let volume = self.volumes[ndims];

        let dims = self.dims.clone();
        let volumes = self.volumes.clone();

        Ok(Shape1::from_parts(ndims, dim, volume, dims, volumes))
    }

    pub fn slice(&self, start: usize, stop: usize) -> ArrResult<Shape1> {
        if start == stop {
            return Err(Error::SliceZeroWidth { index: start });
        } else if start > stop {
            return Err(Error::SliceStopBeforeStart { start, stop });
        } else if stop > self.dim {
            return Err(Error::SliceStopPastEnd {
                stop,
                dim: self.dim,
            });
        }

        let ndims = self.ndims;
        let dim = stop - start;
        let volume = dim * self.volumes[ndims - 1];

        let dims = self.dims.clone();
        let volumes = self.volumes.clone();

        Ok(Shape1::from_parts(ndims, dim, volume, dims, volumes))
    }
}

impl PartialEq for Shape1 {
    fn eq(&self, other: &Self) -> bool {
        self.ndims == other.ndims
            && self.dim == other.dim
            && self.dims[0..self.ndims - 1] == other.dims[0..other.ndims - 1]
    }
}

impl Display for Shape1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.get_dims().reverse())
    }
}
