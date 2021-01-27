mod cursor;
mod dtype;
mod item;
mod iter;
mod nestedlist;
mod result;
mod shape;
use cursor::ArrayCursor;
use dtype::{DType, TypeAware};
use item::Item;
use iter::{ArrayIntoIterator, ArrayIterator, ArrayIteratorMut};
use nestedlist::NestedList;
use result::{Error, ArrayResult};
use shape::Shape;
use std::{cmp, convert, fmt, fs, mem };

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
    fn operate<F, R>(&self, rhs: &Self, op: F) -> ArrayResult<R>
    where
        F: Fn(T, T) -> R,
        R: TypeAware,
    {
        match self.shape().cast(rhs.shape()) {
            Ok(shape) => {
                let mut data = Vec::with_capacity(shape.volume() as usize);

                // TODO: implement once I know what I'm doing

                // 4 x 1 . 1 x 3 => 4 x 3
                // X       X X X    X X X
                // X - >     |      X X X
                // X         v      X X X
                // X                X X X

                // this should work but where do the values
                // even get written to in the array???
                // hypothesis 1: they just get pushed
                // since the order (might) be ordered correctly?
                // also this is a very naive implementation
                // and could definitely be optimized

                // just need a way to access each dimension by index
                // would be nice if we could just access a pointer within
                // the vector and also a starting index
                // or just have a pointer and a counter on how many times to iterate


                fn operate_rec<'a, T, F, R>(a: Item<T>, b: Item<T>, data: &mut Vec<R>, op: &'a F)
                where
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

                let a = Item::new(self);
                let b = Item::new(rhs);

                operate_rec(a, b, &mut data, &op);

                /*
                Possibilities:
                Value and Value => base case
                Arrays of the same rank and same len => linear case
                Arrays of the same rank and different len => stretch where one is 1
                Array and Value => stretch with 1 padding
                Arrays of different rank => stretch with 1 padding

                fn recursive(a: ArrayElement<T>, b: ArrayElement<T>, vec: &mut Vec<R>) {
                    if let (Value(a), Value(b)) = (a, b) {
                        // base case
                    } else if a.ndims() == b.ndims() {
                        if a.len() == b.len() {
                            // linear case
                            for n in ..a.len() {
                                recursive(a.slice(n), b.slice(n), vec);
                            }
                        } else {
                            // stretch where one is 1
                            // this could also probably be simplified
                            if b.len() == 1 {
                                for n in ..a.len() {
                                    recursive(a.slice(n), b.slice(0), vec);
                                }
                            } else {
                                for n in ..b.len() {
                                    recursive(a.slice(0), b.slice(n), vec);
                                }
                            }
                        }
                    } else {
                        // varying ndims (this can probably be simplified)
                        if b.len() == 1 {
                            for n in ..a.len() {
                                recursive(a.slice(n), b, vec);
                            }
                        } else {
                            for n in ..b.len() {
                                recursive(a, b.slice(n), vec);
                            }
                        }
                    }
                }

                // OLD ALGO
                fn recursive(a: Array<T>, b: Array<T>, vec: &mut Vec<R>) {
                    if their ranks are equal {
                        if their ranks are 0 {
                            // base case
                            vec.push(op(a[0], b[0]));
                        } else if their last dims are equal {
                            // linear case
                            let N = a.shape(a.rank()-1);
                            for n in ..N {
                                recursive(a.slice(n), b.slice(n), vec);
                            }
                        } else {
                            // stretch case
                            // N is longer dim (other is 1)
                            let N = a.width * b.width
                            for n in ..N {
                                recursive(a.slice(0), b.slice(n), vec):
                                // where a has width 1 in this case
                            }
                        }
                    } else {
                        // stretch case (where one needs a dimension added)
                        // suppose a has a longer rank
                        let N = a.last_dim();
                        for n in ..N {
                            recursive(a.slice(n), b, vec);
                            // this might be wonky with ownership of b...
                            // also I'm not sure about if this will mess
                            // with the ordering of pushing to the result vec
                        }
                    }
                // where slice() indexes into the array at that index
                }
                */

                Ok(Array { shape, data })
            }
            Err(e) => Err(Error::Cast(e)),
        }
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn rank(&self) -> isize {
        self.shape().ndims()
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

    fn get_data_cursor(&self) -> ArrayCursor<T> {
        ArrayCursor::new(&self.data)
    }

    // fn get_dim_cursor(&'a self) -> ArrayDimCursor<T> {
    //     ArrayDimCursor {
    //         shape: self.shape(),
    //         depth: self.rank() - 1,
    //         index: 0,
    //         cursor: self.get_data_cursor(),
    //     }
    // }

    pub fn reshape(self, shape: Shape) -> ArrayResult<T> {
        if self.shape.volume() == shape.volume() {
            let data = self.iter().copied().collect();
            Ok(Array { shape, data })
        } else {
            Err(Error::Reshape(self.shape().clone(), shape))
        }
    }

    // fn view(&self, index: usize) -> IndexItem<T> {
    //     if self.rank() == 1 {
    //         IndexItem::Val(self.data()[0])
    //     } else {
    //         IndexItem::Arr(Array {
    //             shape: Shape::new(self.shape().iter().take(self.rank() - 1).collect())
    //         })
    //     }
    // }
}

impl<T> cmp::PartialEq for Array<T>
where
    T: TypeAware + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // TODO: change these to self.iter() and other.iter() once ArrayIter is impl'd
        let self_iter = self.data().iter();
        let other_iter = other.data().iter();
        self.rank() == other.rank() && self_iter.zip(other_iter).all(|(&a, &b)| a == b)
    }
}

macro_rules! impl_array_cmp {
    { $( $name:ident: $e:expr ),* } => {
        impl<T> Array<T>
        where
            T: TypeAware + PartialOrd,
        {
            $(
                pub fn $name(&self, rhs: &Self) -> ArrayResult<bool> {
                    self.operate(rhs, $e)
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

macro_rules! impl_array_astype {
    { $( $name:ident for $type_struct:ident as $inner_type:ty ),* } => {
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
    astype_bool for Bool as bool,
    astype_uint8 for Uint8 as u8,
    astype_int8 for Int8 as i8,
    astype_int16 for Int16 as i16,
    astype_int32 for Int32 as i32,
    astype_float32 for Float32 as f32,
    astype_float64 for Float64 as f64
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
        let arr1 = make_array(vec![2,2], vec![0,1,2,3]);
        let arr2 = make_array(vec![2,2], vec![0,1,2,3]);
        assert_eq!(arr1, arr2);
    }

    #[test]
    fn test_cast1() {
        let arr1 = make_array(vec![1], vec![10]);
        let arr2 = make_array(vec![4], vec![0,1,2,3]);
        let expected = make_array(vec![4], vec![0,10,20,30]);

        let op = |a, b| a * b;
        let actual = arr1.operate(&arr2, op).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast2() {
        let arr1 = make_array(vec![1], vec![10]);
        let arr2 = make_array(vec![2, 2], vec![0,1,2,3]);
        let expected = make_array(vec![2, 2], vec![0,10,20,30]);

        let op = |a, b| a * b;
        let actual = arr1.operate(&arr2, op).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast3() {
        let arr1 = make_array(vec![2], vec![0,1]);
        let arr2 = make_array(vec![2,3], vec![0,1,2,3,4,5]);
        let expected = make_array(vec![2,3], vec![0,1,0,3,0,5]);

        let op = |a, b| a * b;
        let actual = arr1.operate(&arr2, op).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cast4() {
        // [[[0 1]
        //   [2 3]]
        //
        //  [[4 5]
        //   [6 7]]]

        // [[0]
        //  [1]]
        let arr1 = make_array(vec![2,2,2], vec![0,1,2,3,4,5,6,7]);
        let arr2 = make_array(vec![1,2], vec![0,1]);
        let expected = make_array(vec![2,2,2], vec![0,0,2,3,0,0,6,7]);

        let op = |a, b| a * b;
        let actual = arr1.operate(&arr2, op).unwrap();

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
