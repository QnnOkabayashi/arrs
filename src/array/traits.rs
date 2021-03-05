use crate::array::{ArrResult, ArrayBase, BroadcastInstruction, Shape};
use core::fmt::Debug;
use core::ops::{Add, Div, Mul, Sub};
use core::slice::Iter;

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

// use a PartialView trait to abstract the idea of taking
// a view into part of a struct
pub trait PartialView<'base> {
    type Base;

    // creates a view of the entire base
    fn from_base(base: &'base Self::Base) -> Self;

    // create a new base from the current view
    fn into_base(&self) -> Self::Base;
}

macro_rules! impl_array_arithmetic {
    { $( $op_name:ident($op_trait:path): $func:expr ),* } => {
        $(
            // can make this work for other T types as well ?
            fn $op_name(&self, other: &Self) -> ArrResult<ArrayBase<<T as $op_trait>::Output>>
            where
                T: $op_trait,
                <T as $op_trait>::Output: TypeAware,
            {
                self.broadcast_combine(other, $func)
            }
        )*
    }
}

macro_rules! impl_array_eq {
    { $( $op_name:ident: $func:expr ),* } => {
        $(
            fn $op_name(&self, other: &Self) -> ArrResult<ArrayBase<bool>>
            where
                T: PartialEq
            {
                self.broadcast_combine(other, $func)
            }
        )*
    }
}

macro_rules! impl_array_ord {
    { $( $op_name:ident: $func:expr ),* } => {
        $(
            fn $op_name(&self, other: &Self) -> ArrResult<ArrayBase<bool>>
            where
                T: PartialOrd
            {
                self.broadcast_combine(other, $func)
            }
        )*
    }
}

// ANY multi dimensional array. Type can be something crazy, hence lack of operations
// The broadcast_combine function is provided to simple array broadcasting, however
pub trait MultiDimensional<'base, T>: PartialView<'base> + Sized + Copy + PartialEq
where
    T: TypeAware,
{
    type SubviewIterator: Iterator<Item = Self>;

    fn ndims(&self) -> usize;

    fn shape(&self) -> &Shape;

    fn one_value(&self) -> T;

    fn iter_values(&self) -> Iter<T>;

    fn one_subview(&self) -> Self;

    fn iter_subviews(&self) -> Self::SubviewIterator;

    fn broadcast_combine<'base2, F, R, A2, T2>(&self, other: &A2, f: F) -> ArrResult<ArrayBase<R>>
    where
        F: Fn(T, T2) -> R,
        R: TypeAware, // return type
        A2: MultiDimensional<'base2, T2>,
        T2: TypeAware, // other type
    {
        match self.shape().broadcast(other.shape()) {
            Ok((shape_base, broadcast_instructions)) => {
                use BroadcastInstruction::*;
                let mut data: Vec<R> = Vec::with_capacity(shape_base.total_volume());

                fn recurse<'base1, 'base2, A1, A2, T1, T2, R, F>(
                    a: A1,
                    b: A2,
                    instructions: &[BroadcastInstruction],
                    data: &mut Vec<R>,
                    f: &F,
                ) where
                    A1: MultiDimensional<'base1, T1>,
                    A2: MultiDimensional<'base2, T2>,
                    T1: TypeAware,
                    T2: TypeAware,
                    R: TypeAware,
                    F: Fn(T1, T2) -> R,
                {
                    let (instruction, sub_instructions) = instructions.split_last().unwrap();
                    match *instruction {
                        PushLinear => {
                            for (&a_value, &b_value) in a.iter_values().zip(b.iter_values()) {
                                data.push(f(a_value, b_value));
                            }
                        }
                        PushStretchA => {
                            let a_value = a.one_value();
                            for &b_value in b.iter_values() {
                                data.push(f(a_value, b_value));
                            }
                        }
                        PushStretchB => {
                            let b_value = b.one_value();
                            for &a_value in a.iter_values() {
                                data.push(f(a_value, b_value));
                            }
                        }
                        RecurseLinear => {
                            for (a_sub, b_sub) in a.iter_subviews().zip(b.iter_subviews()) {
                                recurse(a_sub, b_sub, sub_instructions, data, f);
                            }
                        }
                        RecurseStretchA => {
                            let a_sub = a.one_subview();
                            for b_sub in b.iter_subviews() {
                                recurse(a_sub, b_sub, sub_instructions, data, f);
                            }
                        }
                        RecurseStretchB => {
                            let b_sub = b.one_subview();
                            for a_sub in a.iter_subviews() {
                                recurse(a_sub, b_sub, sub_instructions, data, f);
                            }
                        }
                        RecursePadA => {
                            for b_sub in b.iter_subviews() {
                                recurse(a, b_sub, sub_instructions, data, f);
                            }
                        }
                        RecursePadB => {
                            for a_sub in a.iter_subviews() {
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
        let data = self.iter_values().map(|x| R::from(*x)).collect();

        ArrayBase { shape_base, data }
    }

    impl_array_arithmetic! {
        add_v(Add): |a, b| a + b,
        sub_v(Sub): |a, b| a - b,
        mul_v(Mul): |a, b| a * b,
        div_v(Div): |a, b| a / b
    }

    impl_array_eq! {
        eq_v: |a, b| a == b,
        ne_v: |a, b| a != b
    }

    impl_array_ord! {
        lt_v: |a, b| a < b,
        le_v: |a, b| a <= b,
        gt_v: |a, b| a > b,
        ge_v: |a, b| a >= b
    }
}
