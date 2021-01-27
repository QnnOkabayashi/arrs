mod cursor;
mod dtype;
mod indexitem;
mod iter;
mod nestedlist;
mod result;
mod shape;
use cursor::ArrayCursor;
use dtype::{DType, TypeAware};
use indexitem::IndexItem;
use iter::{ArrayIntoIterator, ArrayIterator, ArrayIteratorMut};
use nestedlist::NestedList;
use result::{ArrayError, ArrayResult};
use shape::Shape;
use std::{convert, fmt, fs, io, mem, rc};

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

                // this seems like such a naive solution
                let lhs_cursor = self.get_data_cursor();
                let rhs_cursor = rhs.get_data_cursor();
                
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
                // but I hate to have to create a struct every time...
                // create a ArrayDimIterator struct that's super bare bones

                fn operate_rec<T, F, R>(lhs: ArrayElement<T>, rhs: ArrayElement<T>, out: &mut Vec<R>, op: F)
                where
                    T: TypeAware,
                    R: TypeAware,
                    F: Fn(T, T) -> R,
                {
                    if let (ArrayElement::Value(a), ArrayElement::Value(b)) = (lhs, rhs) {
                        out.push(op(a, b));
                    }
                }
                /*
                Possibilities:
                Value and Value
                Array and Value
                Arrays of the same rank and same len
                Arrays of the same rank and different len
                Arrays of different rank

                I can probably come up with some way that involves destructuring the enum
                first to be more efficient rather than treating them all the same

                // NEW ALGO PSEUDO CODE
                value & value => base case
                if same rank {
                    // this if statement could probably be compressed
                    if same len {
                        // linear case
                    } else {
                        // stretch where one is 1
                    }
                } else {
                    stretch 1 into n (Array and Value + Arrays of different rank)
                }

                // NEW ALGO
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
            Err(e) => Err(ArrayError::Cast(e)),
        }
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn rank(&self) -> usize {
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
}

impl<T> Array<T>
where
    T: TypeAware + Copy,
{
    pub fn reshape(self, shape: Shape) -> ArrayResult<T> {
        if self.shape.volume() == shape.volume() {
            let data = self.iter().copied().collect();
            Ok(Array { shape, data })
        } else {
            Err(ArrayError::Reshape(self.shape().clone(), shape))
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
    eq: |a, b| a == b,
    ne: |a, b| a != b,
    lt: |a, b| a < b,
    le: |a, b| a <= b,
    gt: |a, b| a > b,
    ge: |a, b| a >= b
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

// iterators over each dimension
// would be nice if I made pseudo-shapes so
// that they match up nicely (equal size, 1-padded)
// a struct where you give it an index, and it gives you
// an iterator for that dim

// rename to "CastingSlice" or something
// so it's clear that it "stretches" if it's a Value varient
pub enum ArrayElement<'a, T>
where
    T: TypeAware,
{
    Array {
        shape: &'a Shape,
        ndims: usize,
        pos: isize,
        cursor: &'a mut ArrayCursor<'a, T>,
    },
    Value(T), // holds a copied value
}

impl<'a, T> ArrayElement<'a, T>
where
    T: TypeAware,
{
    fn ndims(&self) -> usize {
        match self {
            Self::Array { ndims, .. } => *ndims,
            Self::Value(_) => 0,
        }
    }

    fn len(&self) -> isize {
        match self {
            Self::Array { shape, ndims, .. } => shape.dim(ndims - 1),
            Self::Value(_) => 1,
        }
    }

    fn at(&'a self, index: isize) -> Option<ArrayElement<'a, T>> {
        match self {
            Self::Array { shape, ndims, cursor , .. } => {
                if index < shape.dim(ndims - 1) {
                    None
                    // Some(if *ndims == 0 {
                    //     ArrayElement::Value(cursor.read())
                    // } else {
                    //     ArrayElement::Array {
                    //         shape,
                    //         ndims: ndims - 1,
                    //         pos: 0,
                    //         cursor: *cursor,
                    //     }
                    // })
                } else {
                    None
                }
            },
            Self::Value(_) => {
                None
                // Some(self)
            },
        }
    }
}


#[test]
fn test_compile() {
    println!("It works!")
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
    type Error = ArrayError<T>;

    fn try_from(nlist: NestedList<T>) -> ArrayResult<T> {
        todo!()
    }
}

impl<T> convert::TryFrom<fs::File> for Array<T>
where
    T: TypeAware,
{
    type Error = ArrayError<T>;

    fn try_from(f: fs::File) -> ArrayResult<T> {
        todo!()
    }
}
