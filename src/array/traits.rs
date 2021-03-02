// #![feature(const_generics)]
use crate::array::{ArrResult, Error};
use core::fmt::Debug;
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
