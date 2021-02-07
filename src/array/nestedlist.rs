#[derive(Debug)]
pub enum NestedList<T> {
    List(Vec<NestedList<T>>),
    Value(T),
}
