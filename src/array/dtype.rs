pub trait TypeConscious {
    type Type: DType;
}
pub trait DType: std::fmt::Display + PartialEq + Clone + Copy {
    fn new() -> Self;

    fn bytes(&self) -> usize;
}
