use std::fmt;
// impl'd for types that know what type they are
pub trait TypeAware: Copy {
    type Type: DType;
}

// impl'd for unit structs containing type info
pub trait DType: fmt::Display + PartialEq + Clone + Copy {
    fn new() -> Self;

    fn bytes() -> usize;

    fn id() -> u8;
}
