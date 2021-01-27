use super::{dtype::TypeAware, Array};

pub enum IndexItem<T>
where
    T: TypeAware,
{
    Arr(Array<T>),
    Val(T),
}
