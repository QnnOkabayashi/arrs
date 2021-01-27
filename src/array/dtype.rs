// impl'd for types that know what type they are
pub trait TypeAware: Copy {
    type Type: DType;
}

// impl'd for unit structs containing type info
pub trait DType: std::fmt::Display + PartialEq + Clone + Copy {
    fn new() -> Self;

    fn bytes(&self) -> usize;
}
