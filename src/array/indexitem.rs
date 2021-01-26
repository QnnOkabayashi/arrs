use super::{dtype::TypeConscious, Array};

pub enum IndexItem<T>
where
    T: TypeConscious,
{
    Arr(Array<T>),
    Val(T),
}
