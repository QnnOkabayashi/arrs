use super::TypeAware;
use std::cmp::PartialEq;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Data<T>(Vec<T>)
where
    T: TypeAware;

impl<T> Data<T>
where
    T: TypeAware,
{
    pub fn new(data: Vec<T>) -> Self {
        Self(data)
    }

    pub fn as_ptr(&self) -> *const T {
        self.0.as_ptr()
    }
}

impl<T> PartialEq for Data<T>
where
    T: TypeAware + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
        // self.0.len() == other.0.len() && self.0.iter().zip(other.0.iter()).all(|(&a, &b)| a == b)
    }
}
