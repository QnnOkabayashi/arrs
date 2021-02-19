mod data;
mod dtype;
mod error;
mod shape;
mod subarray;
use core::convert::Into;
use core::mem::size_of;
use core::ops;
use core::slice::Iter;
pub use data::Data;
pub use dtype::TypeAware;
pub use error::{ArrResult, Error};
pub use shape::Shape;
use subarray::Subarray;

#[derive(Debug)]
pub struct Array<T>
where
    T: TypeAware,
{
    shape: Shape,
    data: Data<T>,
}

impl<T> Array<T>
where
    T: TypeAware,
{
    pub fn new(shape: Shape, data: Data<T>) -> ArrResult<Self> {
        if shape.volume() != data.len() {
            return Err(Error::ShapeDataMisalignment(shape, data.len()));
        }

        Ok(Self { shape, data })
    }

    fn zip_map<F, R>(&self, other: &Self, op: F) -> ArrResult<Array<R>>
    where
        F: Fn(T, T) -> R,
        R: TypeAware,
    {
        match self.shape().cast(other.shape()) {
            Ok(shape) => {
                let mut raw_data = Vec::with_capacity(shape.volume() as usize);

                fn zip_map_rec<'a, T, R, F>(
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
                                zip_map_rec(a.at(n), b.at(n), data, op);
                            }
                        } else if a.len() == 1 {
                            // stretch where a is 1
                            for n in 0..b.len() {
                                zip_map_rec(a.at(0), b.at(n), data, op);
                            }
                        } else {
                            // stretch where b is 1
                            for n in 0..a.len() {
                                zip_map_rec(a.at(n), b.at(0), data, op);
                            }
                        }
                    } else if a.ndims() < b.ndims() {
                        // stretch where a is 1 padded
                        for n in 0..b.len() {
                            // copies a right now... how bad is this
                            zip_map_rec(a, b.at(n), data, op);
                        }
                    } else {
                        // stretch where b is 1 padded
                        for n in 0..a.len() {
                            zip_map_rec(a.at(n), b, data, op);
                        }
                    }
                }

                zip_map_rec(
                    Subarray::new(self),
                    Subarray::new(other),
                    &mut raw_data,
                    &op,
                );

                let data = Data::new(raw_data);

                Array::new(shape, data)
            }
            Err(e) => Err(e),
        }
    }

    pub fn ndims(&self) -> isize {
        self.shape().ndims()
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn data_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    pub fn iter(&self) -> Iter<T> {
        self.data.iter()
    }

    pub fn reshape(self, shape: Shape) -> ArrResult<Array<T>> {
        if self.shape.volume() == shape.volume() {
            // TODO: once we have special indexing and iterators
            Array::new(shape, self.data)
        } else {
            Err(Error::Reshape(self.shape().clone(), shape))
        }
    }
}

impl<T> PartialEq for Array<T>
where
    T: TypeAware + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // TODO: change these to self.iter() and other.iter() once ArrayIter is impl'd
        self.shape == other.shape && self.data == other.data
    }
}

macro_rules! impl_array_cmp {
    { $( $name:ident: $e:expr ),* } => {
        impl<T> Array<T>
        where
            T: TypeAware + PartialOrd
        {
            $(
                pub fn $name(&self, other: &Self) -> ArrResult<Array<bool>> {
                    self.zip_map(other, $e)
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
                pub fn $name(&self, other: &Self) -> ArrResult<Array<<T as $op_trait>::Output>> {
                    self.zip_map(other, $e)
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
    { $( $name:ident for $inner_type:tt where id: $id:expr),* } => {
        $(
            impl TypeAware for $inner_type {
                const ID: u8 = $id;

                const BYTES: usize = size_of::<$inner_type>();

                const LABEL: &'static str = stringify!($inner_type);
            }

            impl<T> Array<T>
            where
                T: TypeAware + Copy + PartialOrd + Into<$inner_type>,
            {
                pub fn $name(&self) -> Array<$inner_type> {
                    let data = Data::new(self
                        .iter()
                        .map(|x| Into::<$inner_type>::into(*x))
                        .collect::<Vec<$inner_type>>());

                    let shape = self.shape.clone();

                    // not possible for shape and data to be out of sync
                    Array::new(shape, data).unwrap()
                }
            }
        )*
    }
}

impl_array_astype! {
    astype_bool for bool where id: 0x07,
    astype_uint8 for u8 where id: 0x08,
    astype_int8 for i8 where id: 0x09,
    astype_int16 for i16 where id: 0x0B,
    astype_int32 for i32 where id: 0x0C,
    astype_float32 for f32 where id: 0x0D,
    astype_float64 for f64 where id: 0x0E
}
