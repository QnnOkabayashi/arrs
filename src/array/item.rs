use super::{Array, TypeAware, Shape, convert::TryInto};

#[derive(Clone, Copy)]
pub struct Item<'a, T>
where
    T: TypeAware,
{
    shape: &'a Shape,
    ndims: isize,
    offset: isize,
    data: &'a Vec<T>,
}

impl<'a, T> Item<'a, T>
where
    T: TypeAware,
{
    pub fn new(array: &'a Array<T>) -> Self {
        Self {
            shape: array.shape(),
            ndims: array.rank(),
            offset: 0,
            data: array.data(),
        }
    }

    pub fn ndims(&self) -> isize {
        self.ndims
    }

    pub fn len(&self) -> isize {
        self.shape.dim(self.ndims() - 1)
    }

    fn stride(&self) -> isize {
        // NOT THE DIM BELOW, BUT THE PRODUCT OF ALL THE SMALLER DIMS
        // self.shape.dim(self.ndims() - 2)
        let ndims = (self.ndims - 1).try_into().unwrap_or(0);
        self.shape.iter().take(ndims).product()
    }

    pub fn at(&'a self, index: isize) -> Item<'a, T> {
        if 0 <= index && index < self.len() {
            if self.ndims() > 0 {
                Self {
                    shape: self.shape,
                    ndims: self.ndims() - 1,
                    offset: (index * self.stride()) + self.offset,
                    data: self.data,
                }
            } else {
                *self
            }
        } else {
            panic!("index is {}, but len is {}", index, self.len())
        }
    }

    pub fn read(&self) -> T {
        self.data[self.offset as usize]
    }

    // TODO: remake this to support ArrayElement in struct form (not enum)
    // fn at(&'a self, index: isize) -> Option<ArrayElement<'a, T>> {
    //     match self {
    //         Self::Array { shape, ndims, cursor , .. } => {
    //             if index < shape.dim(ndims - 1) {
    //                 None
    //                 // Some(if *ndims == 0 {
    //                 //     ArrayElement::Value(cursor.read())
    //                 // } else {
    //                 //     ArrayElement::Array {
    //                 //         shape,
    //                 //         ndims: ndims - 1,
    //                 //         pos: 0,
    //                 //         cursor: *cursor,
    //                 //     }
    //                 // })
    //             } else {
    //                 None
    //             }
    //         },
    //         Self::Value(_) => {
    //             None
    //             // Some(self)
    //         },
    //     }
    // }
}

