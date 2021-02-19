use crate::array::TypeAware;
use core::slice::Iter;

#[derive(Debug, Clone, PartialEq)]
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

    pub fn iter(&self) -> Iter<T> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
