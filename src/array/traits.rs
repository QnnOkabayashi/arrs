use crate::array::ArrResult;
use core::fmt::Debug;

// impl'd for types that know what type they are
pub trait TypeAware: Copy + PartialEq + Debug {
    // used for idx file format
    const ID: u8;

    // this can be removed
    const BYTES: usize;

    // what to print when asked for dtype
    const LABEL: &'static str;
}

// use a PartialView trait to abstract the idea of taking 
// a view into part of a struct
pub trait PartialView<'base> {
    type Base;

    fn from_base(base: &'base Self::Base) -> Self;

    fn into_base(&self) -> Self::Base;
}
