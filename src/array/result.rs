use super::{dtype::TypeConscious, Array, NestedList};
use super::shape::{CastError, Shape};
use std::{fmt, fs, result};

pub enum ArrayError<T>
where
    T: TypeConscious,
{
    Cast(CastError),
    Reshape(Shape, Shape),
    FromIdxFile(fs::File),
    FromNList(NestedList<T>),
}

impl<T> fmt::Display for ArrayError<T>
where
    T: TypeConscious,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ArrayError: {}",
            match self {
                Self::Cast(CastError(a, b)) => format!(
                    "operands could not be broadcast together with shapes {} {}",
                    a, b
                ),
                Self::Reshape(a, b) => "cannot reshape: shapes don't have same volume".to_string(),
                Self::FromNList(nlist) => "nested list dimensions not consistent".to_string(),
                Self::FromIdxFile(file) => "couldn't create array from file".to_string(),
            }
        )
    }
}

pub type ArrayResult<T> = result::Result<Array<T>, ArrayError<T>>;

mod tests {
    use super::*;
    #[test]
    fn test_broadcast_result() {
        todo!("rewrite these tests");
        // let (a, b) = (vec![1, 2, 3], vec![2, 2, 3]);
        // let err = ArrayError::<i32>::Broadcast(a, b);
        // let res = ArrayResult::Err(err);
        // assert_eq!(res.unwrap_err().to_string(), format!("ArrayError: operands could not be broadcast together with shapes [1, 2, 3] [2, 2, 3]"))
    }

    #[test]
    fn test_tryfrom_nlist_error() {
        let nlist = NestedList::Value(50);
        let err = ArrayError::<i32>::FromNList(nlist);
        assert_eq!(
            format!("{}", err),
            format!("ArrayError: nested list dimensions not consistent")
        )
    }
}
