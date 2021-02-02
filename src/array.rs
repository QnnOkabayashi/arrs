mod dtype;
mod iter;
mod nestedlist;
mod result;
mod shape;
mod subarray;
use dtype::{DType, TypeAware};
use iter::{ArrayIntoIterator, ArrayIterator, ArrayIteratorMut};
use nestedlist::NestedList;
use result::{ArrayResult, Error};
use shape::Shape;
use std::{cmp, convert, fmt, fs, mem, ops};
use subarray::Subarray;

#[derive(Debug)]
pub struct Array<T>
where
    T: TypeAware,
{
    shape: Shape,
    data: Vec<T>,
}

impl<T> Array<T>
where
    T: TypeAware,
{
    fn operate<F, R>(&self, other: &Self, op: F) -> ArrayResult<R>
    where
        F: Fn(T, T) -> R,
        R: TypeAware,
    {
        match self.shape().cast(other.shape()) {
            Ok(shape) => {
                let mut data = Vec::with_capacity(shape.volume() as usize);

                fn operate_rec<'a, T, F, R>(
                    a: Subarray<T>,
                    b: Subarray<T>,
                    data: &mut Vec<R>,
                    op: &'a F,
                ) where
                    T: TypeAware,
                    R: TypeAware,
                    F: Fn(T, T) -> R,
                {
                    if a.ndims() == 0 && b.ndims() == 0 {
                        // base case
                        data.push(op(a.read(), b.read()));
                    } else if a.ndims() == b.ndims() {
                        if a.len() == b.len() {
                            // linear case
                            for n in 0..a.len() {
                                operate_rec(a.at(n), b.at(n), data, op);
                            }
                        } else if a.len() == 1 {
                            // stretch where a is 1
                            for n in 0..b.len() {
                                operate_rec(a.at(0), b.at(n), data, op);
                            }
                        } else {
                            // stretch where b is 1
                            for n in 0..a.len() {
                                operate_rec(a.at(n), b.at(0), data, op);
                            }
                        }
                    } else if a.ndims() < b.ndims() {
                        // stretch where a is 1 padded
                        for n in 0..b.len() {
                            // copies a right now... how bad is this
                            operate_rec(a, b.at(n), data, op);
                        }
                    } else {
                        // stretch where b is 1 padded
                        for n in 0..a.len() {
                            operate_rec(a.at(n), b, data, op);
                        }
                    }
                }

                operate_rec(Subarray::new(self), Subarray::new(other), &mut data, &op);

                Ok(Array { shape, data })
            }
            Err(e) => Err(Error::Cast(e)),
        }
    }

    pub fn ndims(&self) -> isize {
        self.shape().ndims()
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn size(&self) -> isize {
        todo!()
    }

    pub fn len(&self) -> Option<isize> {
        // self.shape.last()
        todo!()
    }

    pub fn dtype() -> T::Type {
        T::Type::new()
    }

    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    fn iter(&self) -> ArrayIterator<T> {
        todo!("create a &T iterator")
    }

    fn iter_mut(&mut self) -> ArrayIteratorMut<T> {
        todo!("create a &mut T iterator")
    }

    // doesn't impl IntoIterator because the flattened version isn't how it should be traversed
    fn into_iter(self) -> ArrayIntoIterator<T> {
        todo!("create a T iterator")
    }

    pub fn reshape(self, shape: Shape) -> ArrayResult<T> {
        if self.shape.volume() == shape.volume() {
            let data = self.iter().copied().collect();
            Ok(Array { shape, data })
        } else {
            Err(Error::Reshape(self.shape().clone(), shape))
        }
    }
}

impl<T> cmp::PartialEq for Array<T>
where
    T: TypeAware + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // TODO: change these to self.iter() and other.iter() once ArrayIter is impl'd
        let self_iter = self.data().iter();
        let other_iter = other.data().iter();
        self.ndims() == other.ndims() && self_iter.zip(other_iter).all(|(&a, &b)| a == b)
    }
}

macro_rules! impl_array_cmp {
    { $( $name:ident: $e:expr ),* } => {
        impl<T> Array<T>
        where
            T: TypeAware + PartialOrd,
        {
            $(
                pub fn $name(&self, other: &Self) -> ArrayResult<bool> {
                    self.operate(other, $e)
                }
            )*
        }
    }
}

impl_array_cmp! {
    v_eq: |a, b| a == b,
    v_ne: |a, b| a != b,
    v_lt: |a, b| a < b,
    v_le: |a, b| a <= b,
    v_gt: |a, b| a > b,
    v_ge: |a, b| a >= b
}

macro_rules! impl_array_op {
    { $( $name:ident($op_trait:path): $e:expr ),* } => {
        $(
            impl<T> Array<T>
            where
            T: TypeAware + $op_trait,
            <T as $op_trait>::Output: TypeAware,
            {
                pub fn $name(&self, other: &Self) -> ArrayResult<<T as $op_trait>::Output> {
                    self.operate(other, $e)
                }
            }
        )*
    }
}

impl_array_op! {
    v_add(ops::Add): |a, b| a + b,
    v_sub(ops::Sub): |a, b| a - b,
    v_mul(ops::Mul): |a, b| a * b,
    v_div(ops::Div): |a, b| a / b,
    v_rem(ops::Rem): |a, b| a % b
}

macro_rules! impl_array_astype {
    { $( $name:ident for $inner_type:ty as $type_struct:ident ),* } => {
        $(
            #[derive(Copy, Clone, PartialEq)]
            pub struct $type_struct;

            impl DType for $type_struct {
                fn new() -> Self {
                    $type_struct
                }

                fn bytes(&self) -> usize {
                    mem::size_of::<$inner_type>()
                }
            }

            impl fmt::Display for $type_struct {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, stringify!($type_struct))
                }
            }

            impl TypeAware for $inner_type {
                type Type = $type_struct;
            }

            impl<T> Array<T>
            where
                T: TypeAware + Copy + PartialOrd + convert::Into<$inner_type>,
            {
                pub fn $name(&self) -> Array<$inner_type> {
                    let data = self
                        .iter()
                        .map(|x| convert::Into::<$inner_type>::into(*x))
                        .collect::<Vec<$inner_type>>();

                    Array {
                        shape: self.shape.clone(),
                        data,
                    }
                }
            }
        )*
    }
}

impl_array_astype! {
    astype_bool for bool as Bool,
    astype_uint8 for u8 as Uint8,
    astype_int8 for i8 as Int8,
    astype_int16 for i16 as Int16,
    astype_int32 for i32 as Int32,
    astype_float32 for f32 as Float32,
    astype_float64 for f64 as Float64
}

mod tests {
    use super::*;

    fn make_array<T>(shape: Vec<isize>, data: Vec<T>) -> Array<T>
    where
        T: TypeAware,
    {
        let shape = Shape::new(shape);
        assert!(shape.volume() == data.len() as isize);
        Array { shape, data }
    }

    #[test]
    fn test_eq1() {
        let arr1 = make_array(vec![2, 2], vec![0, 1, 2, 3]);
        let arr2 = make_array(vec![2, 2], vec![0, 1, 2, 3]);
        assert_eq!(arr1, arr2);
    }

    #[test]
    fn test_cast1() {
        let arr1 = make_array(vec![1], vec![10]);
        let arr2 = make_array(vec![4], vec![0, 1, 2, 3]);
        let expected = make_array(vec![4], vec![0, 10, 20, 30]);

        let op = |a, b| a * b;
        let actual = arr1.operate(&arr2, op).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast2() {
        let arr1 = make_array(vec![1], vec![10]);
        let arr2 = make_array(vec![2, 2], vec![0, 1, 2, 3]);
        let expected = make_array(vec![2, 2], vec![0, 10, 20, 30]);

        let op = |a, b| a * b;
        let actual = arr1.operate(&arr2, op).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast3() {
        let arr1 = make_array(vec![2], vec![0, 1]);
        let arr2 = make_array(vec![2, 3], vec![0, 1, 2, 3, 4, 5]);
        let expected = make_array(vec![2, 3], vec![0, 1, 0, 3, 0, 5]);

        let op = |a, b| a * b;
        let actual = arr1.operate(&arr2, op).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast4() {
        let arr1 = make_array(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let arr2 = make_array(vec![1, 2], vec![0, 1]);
        let expected = make_array(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]);

        let op = |a, b| a * b;
        let actual = arr1.operate(&arr2, op).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast5() {
        let arr1 = make_array(vec![2, 2, 2], vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let arr2 = make_array(vec![1, 2], vec![0, 1]);
        let expected = make_array(vec![2, 2, 2], vec![0, 0, 2, 3, 0, 0, 6, 7]);

        let actual = arr1.v_mul(&arr2).unwrap();

        assert_eq!(expected, actual);
    }
}

impl<T> convert::From<Vec<T>> for Array<T>
where
    T: TypeAware,
{
    fn from(data: Vec<T>) -> Self {
        let shape = Shape::new(vec![1]);
        Self { shape, data }
    }
}

impl<T> convert::TryFrom<NestedList<T>> for Array<T>
where
    T: TypeAware,
{
    type Error = Error<T>;

    fn try_from(nlist: NestedList<T>) -> ArrayResult<T> {
        todo!()
    }
}

impl<T> convert::TryFrom<fs::File> for Array<T>
where
    T: TypeAware,
{
    type Error = Error<T>;

    fn try_from(f: fs::File) -> ArrayResult<T> {
        todo!()
    }
}
