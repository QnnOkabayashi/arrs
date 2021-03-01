mod error;
mod shape;
mod subarray;
mod traits;
use core::cmp::PartialEq;
use core::convert::Into;
use core::mem::size_of;
use core::ops;
use core::slice::Iter;
pub use error::{ArrResult, Error};
pub use shape::{Shape, Shape1, ShapeBase};
use shape::BroadcastInstruction;
use std::{fmt::Debug, sync::Arc};
use subarray::Subarray;
pub use traits::{TypeAware, PartialView};

// to embed, use an Rc instead depending on no_std
// https://www.reddit.com/r/rust/comments/49hlsf/no_std_library_optionally_depending_onusing_std/
type Data<T> = Arc<Vec<T>>;

pub struct ArrayBase<T: TypeAware> {
    shape_base: ShapeBase,
    data: Vec<T>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
// DenseArray is a PartialView of ArrayBase where elements
// are stored contiguously in memory. This means fast 
// but restrictive array indexing
pub struct DenseArray<'base, T: TypeAware> {
    shape: Shape<'base>,
    data: &'base [T],
}

impl<T: TypeAware> ArrayBase<T> {
    pub fn new_checked(dims: Vec<usize>, data: Vec<T>) -> ArrResult<Self> {
        let shape_base = ShapeBase::new_checked(dims)?;

        if shape_base.total_volume() != data.len() {
            return Err(Error::ShapeDataMisalignment {
                shape_volume: shape_base.total_volume(),
                data_len: data.len(),
            });
        }

        Ok(Self { shape_base, data })
    }
}

// struct SparseArray; // dynamic access patterns

impl<'a, T: TypeAware> DenseArray<'a, T> {
    pub fn ndims(&self) -> usize {
        self.shape.ndims()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    fn at(&self, index: usize) -> T {
        // todo: implement Index trait later
        // keep this for clarity rn
        self.data[index]
    }

    // instead of having an `at` method, wouldn't it make
    // more sense to just have a method to iterate over
    // the elements? more compiler optimizations probably

    pub fn at_checked(&self, index: usize) -> ArrResult<T> {
        if self.ndims() > 1 {
            return Err(Error::ReadNDim {
                ndims: self.ndims(),
            });
        } else if index >= self.len() {
            return Err(Error::DerankIndexOutOfBounds {
                len: self.len(),
                index,
            });
        }

        Ok(self.at(index))
    }

    pub fn derank(&'a self, index: usize) -> ArrResult<Self> {
        // TODO: make iterate through deranked instead
        // of accessing them one by one
        let shape = self.shape.derank_checked(index)?;
        let step = self.shape.stride();
        let data = &self.data[index * step..(index + 1) * step];

        Ok(Self { shape, data })
    }

    pub fn slice(&'a self, start: usize, stop: usize) -> ArrResult<Self> {
        let shape = self.shape.slice_checked(start, stop)?;
        let step = self.shape.stride();
        let data = &self.data[step * start..step * stop];

        Ok(Self { shape, data })
    }

    fn broadcast_merge<F, R>(&self, other: &Self, op: F) -> ArrResult<ArrayBase<R>>
    where
        F: Fn(T, T) -> R,
        R: TypeAware,
    {
        match self.shape.broadcast(&other.shape) {
            Ok((shape_base, broadcast_instructions)) => {
                use BroadcastInstruction::*;
                let mut data: Vec<R> = Vec::with_capacity(shape_base.total_volume());

                fn recurse<T, R, F>(
                    a: DenseArray<T>,
                    b: DenseArray<T>,
                    instructions: &[BroadcastInstruction],
                    data: &mut Vec<R>,
                    op: &F,
                ) -> ArrResult<()>
                where
                    T: TypeAware,
                    R: TypeAware,
                    F: Fn(T, T) -> R,
                {
                    let (instruction, sub_instructions) = instructions.split_last().unwrap();
                    match *instruction {
                        PushLinear => {
                            for (&a_value, &b_value) in a.data.iter().zip(b.data.iter()) {
                                data.push(op(a_value, b_value));
                            }
                        }
                        PushStretchA => {
                            for &b_value in b.data.iter() {
                                data.push(op(a.at_checked(0)?, b_value));
                            }
                        }
                        PushStretchB => {
                            for &a_value in a.data.iter() {
                                data.push(op(a_value, b.at_checked(0)?));
                            }
                        }
                        RecurseLinear => {
                            for index in 0..a.shape.len() {
                                recurse(a.derank(index)?, b.derank(index)?, sub_instructions, data, op)?;
                            }
                        }
                        RecurseStretchA => {
                            for i in 0..b.shape.len() {
                                recurse(a.clone(), b.slice(i, i+1)?, sub_instructions, data, op)?;
                            }
                        }
                        RecurseStretchB => {
                            for i in 0..a.shape.len() {
                                recurse(a.slice(i, i+1)?, b.clone(), sub_instructions, data, op)?;
                            }
                        }
                        RecursePadA => {
                            for index in 0..b.shape.len() {
                                recurse(a.clone(), b.derank(index)?, sub_instructions, data, op)?;
                            }
                        }
                        RecursePadB => {
                            for index in 0..a.shape.len() {
                                recurse(a.derank(index)?, b.clone(), sub_instructions, data, op)?;
                            }
                        }
                    }
                    Ok(())
                }

                recurse(self.clone(), other.clone(), &broadcast_instructions, &mut data, &op)?;

                Ok(ArrayBase { shape_base, data })
            }
            Err(e) => Err(e),
        }
    }
}

impl<'base, T: TypeAware> PartialView<'base> for DenseArray<'base, T> {
    type Base = ArrayBase<T>;

    fn from_base(base: &'base Self::Base) -> Self {
        Self {
            shape: Shape::from_base(&base.shape_base),
            data: &base.data[..],
        }
    }

    fn into_base(&self) -> Self::Base {
        ArrayBase {
            shape_base: self.shape.into_base(),
            data: self.data.to_vec(),
        }
    }
}

macro_rules! impl_array_cmp {
    { $( $name:ident: $e:expr ),* } => {
        impl<'a, T> DenseArray<'a, T>
        where
            T: TypeAware + PartialOrd
        {
            $(
                pub fn $name(&self, other: &Self) -> ArrResult<ArrayBase<bool>> {
                    self.broadcast_merge(other, $e)
                }
            )*
        }
    }
}

macro_rules! impl_array_op {
    { $( $op:ident($op_trait:path): $func:expr ),* } => {
        $(
            impl<'a, T> DenseArray<'a, T>
            where
                T: TypeAware + $op_trait,
                <T as $op_trait>::Output: TypeAware,
            {
                pub fn $op(&self, other: &Self) -> ArrResult<ArrayBase<<T as $op_trait>::Output>> {
                    self.broadcast_merge(other, $func)
                }
            }
        )*
    }
}

macro_rules! impl_array_astype {
    { $( $name:ident for $inner_type:tt where id: $id:expr),* } => {
        $(
            impl TypeAware for $inner_type {
                const ID: u8 = $id;

                const BYTES: usize = size_of::<$inner_type>();

                const LABEL: &'static str = stringify!($inner_type);
            }

            impl<'a, T> DenseArray<'a, T>
            where
                T: TypeAware + Into<$inner_type>,
            {
                pub fn $name(&self) -> ArrayBase<$inner_type> {
                    let shape_base = self.shape.into_base();
                    let data = self.data.iter().map(|x| Into::<$inner_type>::into(*x)).collect();

                    ArrayBase{ shape_base, data }
                }
            }
            // do ArrayMut here later
        )*
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

impl_array_op! {
    add(ops::Add): |a, b| a + b,
    sub(ops::Sub): |a, b| a - b,
    mul(ops::Mul): |a, b| a * b,
    div(ops::Div): |a, b| a / b,
    rem(ops::Rem): |a, b| a % b
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

#[derive(Debug, Clone)]
pub struct Array1<T>
where
    T: TypeAware,
{
    shape: Shape1,
    data: Data<T>,
    offset: usize,
}

impl<T> Array1<T>
where
    T: TypeAware,
{
    pub fn new(shape: Shape1, data: Data<T>) -> ArrResult<Self> {
        if shape.volume() != data.len() {
            return Err(Error::ShapeDataMisalignment1 {
                shape,
                data_len: data.len(),
            });
        }

        Ok(Self::from_parts(shape, data, 0))
    }

    fn from_parts(shape: Shape1, data: Data<T>, offset: usize) -> Self {
        Self {
            shape,
            data,
            offset,
        }
    }

    // can probably rewrite most of this once slicing is implemented correctly
    fn zip_map<F, R>(&self, other: &Self, op: F) -> ArrResult<Array1<R>>
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

                Ok(Array1::from_parts(shape, data, 0))
            }
            Err(e) => Err(e),
        }
    }

    pub fn ndims(&self) -> usize {
        self.shape().ndims()
    }

    pub fn shape(&self) -> &Shape1 {
        &self.shape
    }

    pub fn len(&self) -> usize {
        self.shape().len()
    }

    pub fn data_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    pub fn iter(&self) -> Iter<T> {
        self.data.iter()
    }

    pub fn reshape(self, shape: Shape1) -> ArrResult<Self> {
        // rewrite later
        if self.shape.volume() == shape.volume() {
            // TODO: once we have special indexing and iterators
            // data is all continuous so shouldn't be hard at all
            Array1::new(shape, self.data)
        } else {
            Err(Error::Reshape {
                initial: self.shape().clone(),
                target: shape,
            })
        }
    }

    pub fn get_checked(&self, index: usize) -> ArrResult<T> {
        if self.ndims() > 1 {
            return Err(Error::ReadNDim {
                ndims: self.ndims(),
            });
        } else if index >= self.len() {
            return Err(Error::DerankIndexOutOfBounds {
                len: self.len(),
                index,
            });
        }

        Ok(self.get(index))
    }

    fn get(&self, index: usize) -> T {
        // panics if index is out of range
        self.data[self.offset + index]
    }

    pub fn derank(&self, index: usize) -> ArrResult<Self> {
        let shape = self.shape().derank(index)?;
        let data = self.data.clone();
        let offset = self.shape().inside_volume() * index + self.offset;

        Ok(Self::from_parts(shape, data, offset))
    }

    pub fn slice(&self, start: usize, stop: usize) -> ArrResult<Self> {
        let shape = self.shape().slice(start, stop)?;
        let data = self.data.clone();
        let offset = self.shape().inside_volume() * start + self.offset;

        Ok(Self::from_parts(shape, data, offset))
    }

    pub fn fresh_copy(&self) -> Self {
        // there is no performance benefit to creating a fresh copy!
        // only do it if you don't want changes reflected in the original
        let shape = self.shape.clone();
        let data = Arc::new(self.data[self.offset..shape.volume()].to_vec());

        Self::from_parts(shape, data, 0)
    }
}

impl<T> PartialEq for Array1<T>
where
    T: TypeAware,
{
    fn eq(&self, other: &Self) -> bool {
        self.shape() == other.shape()
            && self.data[self.offset..self.offset + self.shape().volume()]
                == other.data[other.offset..other.offset + other.shape().volume()]
    }
}

macro_rules! impl_array_cmp1 {
    { $( $name:ident: $e:expr ),* } => {
        impl<T> Array1<T>
        where
            T: TypeAware + PartialOrd
        {
            $(
                pub fn $name(&self, other: &Self) -> ArrResult<Array1<bool>> {
                    self.zip_map(other, $e)
                }
            )*
        }
    }
}

macro_rules! impl_array_op1 {
    { $( $name:ident($op_trait:path): $e:expr ),* } => {
        $(
            impl<T> Array1<T>
            where
            T: TypeAware + $op_trait,
            <T as $op_trait>::Output: TypeAware,
            {
                pub fn $name(&self, other: &Self) -> ArrResult<Array1<<T as $op_trait>::Output>> {
                    self.zip_map(other, $e)
                }
            }
        )*
    }
}

macro_rules! impl_array_astype1 {
    { $( $name:ident for $inner_type:tt where id: $id:expr),* } => {
        $(
            // impl TypeAware for $inner_type {
            //     const ID: u8 = $id;

            //     const BYTES: usize = size_of::<$inner_type>();

            //     const LABEL: &'static str = stringify!($inner_type);
            // }

            impl<T> Array1<T>
            where
                T: TypeAware + Copy + PartialOrd + Into<$inner_type>,
            {
                pub fn $name(&self) -> Array1<$inner_type> {
                    let data = Data::new(self
                        .iter()
                        .map(|x| Into::<$inner_type>::into(*x))
                        .collect::<Vec<$inner_type>>());

                    let shape = self.shape.clone();

                    // not possible for shape and data to be out of sync
                    Array1::new(shape, data).unwrap()
                }
            }
        )*
    }
}

impl_array_cmp1! {
    v_eq: |a, b| a == b,
    v_ne: |a, b| a != b,
    v_lt: |a, b| a < b,
    v_le: |a, b| a <= b,
    v_gt: |a, b| a > b,
    v_ge: |a, b| a >= b
}

impl_array_op1! {
    v_add(ops::Add): |a, b| a + b,
    v_sub(ops::Sub): |a, b| a - b,
    v_mul(ops::Mul): |a, b| a * b,
    v_div(ops::Div): |a, b| a / b,
    v_rem(ops::Rem): |a, b| a % b
}

impl_array_astype1! {
    astype_bool for bool where id: 0x07,
    astype_uint8 for u8 where id: 0x08,
    astype_int8 for i8 where id: 0x09,
    astype_int16 for i16 where id: 0x0B,
    astype_int32 for i32 where id: 0x0C,
    astype_float32 for f32 where id: 0x0D,
    astype_float64 for f64 where id: 0x0E
}
