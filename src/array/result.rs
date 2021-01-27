use super::shape::{CastError, Shape};
use super::{dtype::TypeAware, Array, NestedList};
use std::{fmt, fs, result};

#[derive(Debug)]
pub enum Error<T>
where
    T: TypeAware,
{
    Cast(CastError),
    Reshape(Shape, Shape),
    FromIdxFile(fs::File),
    FromNList(NestedList<T>),
}

impl<T> fmt::Display for Error<T>
where
    T: TypeAware,
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
                Self::Reshape(a, b) => format!(
                    "cannot reshape: shapes have different volumes (a: {:?} -> {}, b: {:?} -> {})",
                    a,
                    a.volume(),
                    b,
                    b.volume()
                ),
                Self::FromNList(_) => "nested list dimensions not consistent".to_string(),
                Self::FromIdxFile(_) => "couldn't create array from file".to_string(),
            }
        )
    }
}

pub type ArrayResult<T> = result::Result<Array<T>, Error<T>>;

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
        let err = Error::<i32>::FromNList(nlist);
        assert_eq!(
            format!("{}", err),
            format!("ArrayError: nested list dimensions not consistent")
        )
    }
}
