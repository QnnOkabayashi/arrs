use crate::array::{Arc, ArrResult, Error};
use core::{
    cmp::max,
    fmt::{self, Display},
    iter::once,
};

// lowest dimensions are first
#[derive(Debug, Clone)]
pub struct Shape {
    ndims: usize,
    dim: usize,
    volume: usize,
    dims: Arc<Vec<usize>>,
    volumes: Arc<Vec<usize>>,
}

impl Shape {
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

    pub fn cast(&self, other: &Shape) -> ArrResult<Shape> {
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
        Ok(Shape::new(dims))
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

    pub fn derank(&self, index: usize) -> ArrResult<Shape> {
        if self.ndims() == 1 {
            return Err(Error::Derank1D);
        } else if index >= self.dim {
            return Err(Error::DerankInvalidIndex {
                len: self.dim,
                index,
            });
        }

        let ndims = self.ndims - 1;
        let dim = self.dims[ndims - 1];
        let volume = self.volumes[ndims];

        let dims = self.dims.clone();
        let volumes = self.volumes.clone();

        Ok(Shape::from_parts(ndims, dim, volume, dims, volumes))
    }

    pub fn slice(&self, start: usize, stop: usize) -> ArrResult<Shape> {
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

        Ok(Shape::from_parts(ndims, dim, volume, dims, volumes))
    }
}

impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        self.ndims == other.ndims
            && self.dim == other.dim
            && self.dims[0..self.ndims - 1] == other.dims[0..other.ndims - 1]
    }
}

impl Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.get_dims().reverse())
    }
}
