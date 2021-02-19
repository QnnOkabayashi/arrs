use std::{cmp, fmt, result};
use serde::{Serialize, Deserialize};

// lowest dimensions are first
#[derive(Debug, Clone, Deserialize)]
pub struct Shape(Vec<isize>);

impl Shape {
    pub fn new(dims: Vec<isize>) -> Self {
        assert!(dims.len() > 0, "shapes can't have 0 dims");
        // probably should return a `Result` type because it's not impossible
        Shape(dims)
    }

    pub fn cast(&self, other: &Shape) -> CastResult {
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
                _ => return Err(CastError(self.clone(), other.clone())),
            })
        }

        Ok(Shape::new(dims))
    }

    pub fn ndims(&self) -> isize {
        self.0.len() as isize
    }

    pub fn dim(&self, index: isize) -> isize {
        assert!(0 <= index && index < self.ndims(), "index: {}, ndims: {}", index, self.ndims());
        self.0[index as usize]
    }

    pub fn volume(&self) -> isize {
        self.iter().product()
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

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}",
            self.0.iter().rev().copied().collect::<Vec<isize>>()
        )
    }
}

// since casting is completely independent of array types,
// we need to define our own Error so it doesn't need to be
// a generic type
#[derive(Debug, PartialEq)]
pub struct CastError(pub Shape, pub Shape);

pub type CastResult = result::Result<Shape, CastError>;

mod tests {
    use super::*;

    #[test]
    fn test_eq1() {
        let a = Shape::new(vec![3, 4, 5]);
        let b = Shape::new(vec![3, 4, 5]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_eq2() {
        let a = Shape::new(vec![1000, 1000, 1000]);
        let b = Shape::new(vec![1000, 1000, 1000]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_ne1() {
        let a = Shape::new(vec![1000, 1000, 1000]);
        let b = Shape::new(vec![1001, 1000, 1000]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_ne2() {
        let a = Shape::new(vec![1, 2, 3]);
        let b = Shape::new(vec![1, 2]);
        assert_ne!(a, b);
    }

    #[test]
    fn test_cast_ok1() {
        let a = Shape::new(vec![3, 256, 256]);
        let b = Shape::new(vec![3]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![3, 256, 256]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok2() {
        let a = Shape::new(vec![1, 6, 1, 8]);
        let b = Shape::new(vec![5, 1, 7]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![5, 6, 7, 8]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok3() {
        let a = Shape::new(vec![4, 5]);
        let b = Shape::new(vec![1]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![4, 5]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok4() {
        let a = Shape::new(vec![4, 5]);
        let b = Shape::new(vec![4]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![4, 5]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok5() {
        let a = Shape::new(vec![5, 3, 15]);
        let b = Shape::new(vec![5, 1, 15]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![5, 3, 15]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok6() {
        let a = Shape::new(vec![5, 3, 15]);
        let b = Shape::new(vec![5, 3]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![5, 3, 15]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_ok7() {
        let a = Shape::new(vec![5, 3, 15]);
        let b = Shape::new(vec![1, 3]);
        let res = a.cast(&b);
        let expected = Shape::new(vec![5, 3, 15]);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_cast_err1() {
        let a = Shape::new(vec![3]);
        let b = Shape::new(vec![4]);
        let res = a.cast(&b);
        let expected = CastError(a.clone(), b.clone());
        assert_eq!(expected, res.unwrap_err());
    }

    #[test]
    fn test_cast_err2() {
        let a = Shape::new(vec![1, 2]);
        let b = Shape::new(vec![3, 4, 8]);
        let res = a.cast(&b);
        let expected = CastError(a.clone(), b.clone());
        assert_eq!(expected, res.unwrap_err());
    }

    #[test]
    fn test_volume1() {
        let a = Shape::new(vec![3, 256, 256]);
        let expected = 3 * 256 * 256 as isize;
        assert_eq!(expected, a.volume());
    }

    #[test]
    fn test_volume2() {
        let a = Shape::new((1..11).collect::<Vec<isize>>());
        let expected = (1..11).product::<isize>();
        assert_eq!(expected, a.volume());
    }

    #[test]
    fn test_display1() {
        let a = Shape::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(a.to_string(), String::from("[5, 4, 3, 2, 1]"));
    }
}
