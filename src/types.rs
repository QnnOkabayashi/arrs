use core::fmt::Debug;
use core::iter::Sum;
use core::ops::{Add, Div, Mul, Sub};

pub trait ArrType:
    Copy
    + PartialEq
    + Debug
    + Sum
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
}

impl ArrType for u8 {}
impl ArrType for i8 {}
impl ArrType for i16 {}
impl ArrType for i32 {}
impl ArrType for f32 {}
impl ArrType for f64 {}
