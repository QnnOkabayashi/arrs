use crate::array::{ArrResult, Error};
use std::{
    cmp,
    fmt::{self, Display},
};

// lowest dimensions are first
#[derive(Debug, Clone)]
pub struct Shape(Vec<isize>);

impl Shape {
    pub fn new(dims: Vec<isize>) -> Self {
        assert!(dims.len() > 0, "shapes can't have 0 dims");
        // probably should return a `Result` type because it's not impossible
        Shape(dims)
    }

    pub fn cast(&self, other: &Shape) -> ArrResult<Shape> {
        let mut dims = Vec::with_capacity(cmp::max(self.ndims(), other.ndims()) as usize);

        let (mut lhs, mut rhs) = (self.iter(), other.iter());

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
                _ => return Err(Error::Cast(self.clone(), other.clone())),
            })
        }

        Ok(Shape::new(dims))
    }

    pub fn ndims(&self) -> isize {
        self.0.len() as isize
    }

    pub fn dim(&self, index: isize) -> isize {
        assert!(
            0 <= index && index < self.ndims(),
            "index: {}, ndims: {}",
            index,
            self.ndims()
        );
        self.0[index as usize]
    }

    pub fn volume(&self) -> usize {
        self.iter().product::<isize>() as usize
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &isize> {
        self.0.iter()
    }
}

impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
        // self.ndims() == rhs.ndims() && self.iter().zip(rhs.iter()).all(|(a, b)| a == b)
    }
}

impl Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}",
            self.0.iter().rev().copied().collect::<Vec<isize>>()
        )
    }
}
