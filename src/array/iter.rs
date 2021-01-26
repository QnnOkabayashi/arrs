use super::{dtype::TypeConscious, Array};
use std::iter::Iterator;

pub struct ArrayIterator<'a, T>
where
    T: TypeConscious,
{
    base: &'a Array<T>,
    index: usize,
}

pub struct ArrayIteratorMut<'a, T>
where
    T: TypeConscious,
{
    base: &'a mut Array<T>,
    index: usize,
}

pub struct ArrayIntoIterator<T>
where
    T: TypeConscious,
{
    base: Array<T>,
    index: usize,
}

impl<'a, T> Iterator for ArrayIterator<'a, T>
where
    T: TypeConscious,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a, T> Iterator for ArrayIteratorMut<'a, T>
where
    T: TypeConscious,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<T> Iterator for ArrayIntoIterator<T>
where
    T: TypeConscious,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
