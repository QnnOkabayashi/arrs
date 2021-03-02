use crate::array::{ArrResult, ArrayBase, BroadcastInstruction, Error, Shape};
use core::fmt::Debug;
use core::slice::Iter;
use std::io::{Read, Write};

// impl'd for types that know what type they are
pub trait TypeAware: Copy + PartialEq + Debug {
    const LABEL: &'static str;
}

macro_rules! impl_typeaware {
    { $( $inner_type:tt ),* } => {
        $(
            impl TypeAware for $inner_type {
                const LABEL: &'static str = stringify!($inner_type);
            }
        )*
    }
}

impl_typeaware! { bool, u8, i8, i16, i32, f32, f64 }

pub trait IdxType: TypeAware {
    const ID: u8;

    fn read<R: Read>(reader: &mut R) -> ArrResult<Self>;

    fn write<W: Write>(&self, writer: &mut W) -> ArrResult<()>;
}

macro_rules! impl_idxtype {
    { $( $inner_type:tt, $size:expr, $id:expr),* } => {
        $(
            impl IdxType for $inner_type {
                const ID: u8 = $id;

                fn read<R: Read>(reader: &mut R) -> ArrResult<Self> {
                    let mut buf = [0; $size];
                    if reader.read(&mut buf)? < $size {
                        return Err(Error::IdxReadUnaccepted);
                    }

                    Ok(Self::from_be_bytes(buf))
                }

                fn write<W: Write>(&self, writer: &mut W) -> ArrResult<()> {
                    let buf = self.to_be_bytes();
                    if writer.write(&buf)? < $size {
                        return Err(Error::IdxWriteUnaccepted);
                    }

                    Ok(())
                }
            }
        )*
    }
}

impl_idxtype! {
    u8, 1, 0x08,
    i8, 1, 0x09,
    i16, 2, 0x0B,
    i32, 4, 0x0C,
    f32, 4, 0x0D,
    f64, 8, 0x0E
}

// use a PartialView trait to abstract the idea of taking
// a view into part of a struct
pub trait PartialView<'base> {
    type Base;

    // creates a view of the entire base
    fn from_base(base: &'base Self::Base) -> Self;

    // create a new base from the current view
    fn into_base(&self) -> Self::Base;
}

pub trait Broadcastable<'base, T: TypeAware>: PartialView<'base> + Sized + Copy {
    type SubIterator: Iterator<Item = Self>;

    fn one_data(&self) -> T;

    fn iter_flat_data(&self) -> Iter<T>;

    fn one_subarray(&self) -> Self;

    fn iter_subarray(&self) -> Self::SubIterator;

    fn shape(&self) -> &Shape;

    // TODO: make this take any other type that's also broadcastable
    fn broadcast_combine<F, R>(&self, other: &Self, f: F) -> ArrResult<ArrayBase<R>>
    where
        F: Fn(T, T) -> R,
        R: TypeAware,
    {
        match self.shape().broadcast(other.shape()) {
            Ok((shape_base, broadcast_instructions)) => {
                use BroadcastInstruction::*;
                let mut data: Vec<R> = Vec::with_capacity(shape_base.total_volume());

                fn recurse<'base, B, T, R, F>(
                    a: B,
                    b: B,
                    instructions: &[BroadcastInstruction],
                    data: &mut Vec<R>,
                    f: &F,
                ) where
                    B: Broadcastable<'base, T>,
                    T: TypeAware,
                    R: TypeAware,
                    F: Fn(T, T) -> R,
                {
                    let (instruction, sub_instructions) = instructions.split_last().unwrap();
                    match *instruction {
                        PushLinear => {
                            for (&a_value, &b_value) in a.iter_flat_data().zip(b.iter_flat_data()) {
                                data.push(f(a_value, b_value));
                            }
                        }
                        PushStretchA => {
                            let a_value = a.one_data();
                            for &b_value in b.iter_flat_data() {
                                data.push(f(a_value, b_value));
                            }
                        }
                        PushStretchB => {
                            let b_value = b.one_data();
                            for &a_value in a.iter_flat_data() {
                                data.push(f(a_value, b_value));
                            }
                        }
                        RecurseLinear => {
                            for (a_sub, b_sub) in a.iter_subarray().zip(b.iter_subarray()) {
                                recurse(a_sub, b_sub, sub_instructions, data, f);
                            }
                        }
                        RecurseStretchA => {
                            let a_sub = a.one_subarray();
                            for b_sub in b.iter_subarray() {
                                recurse(a_sub, b_sub, sub_instructions, data, f);
                            }
                        }
                        RecurseStretchB => {
                            let b_sub = b.one_subarray();
                            for a_sub in a.iter_subarray() {
                                recurse(a_sub, b_sub, sub_instructions, data, f);
                            }
                        }
                        RecursePadA => {
                            for b_sub in b.iter_subarray() {
                                recurse(a, b_sub, sub_instructions, data, f);
                            }
                        }
                        RecursePadB => {
                            for a_sub in a.iter_subarray() {
                                recurse(a_sub, b, sub_instructions, data, f);
                            }
                        }
                    }
                }

                recurse(*self, *other, &broadcast_instructions, &mut data, &f);

                Ok(ArrayBase { shape_base, data })
            }
            Err(e) => Err(e),
        }
    }

    fn as_type<R: TypeAware + From<T>>(&self) -> ArrayBase<R> {
        let shape_base = self.shape().into_base();
        let data = self.iter_flat_data().map(|x| R::from(*x)).collect();

        ArrayBase { shape_base, data }
    }
}
