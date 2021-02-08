use super::{Array, Shape, TypeAware};

#[derive(Clone, Copy)]
pub struct Subarray<'a, T>
where
    T: TypeAware,
{
    shape: &'a Shape,
    ndims: isize,
    data: *const T,
}

impl<'a, T> Subarray<'a, T>
where
    T: TypeAware,
{
    pub fn new(array: &'a Array<T>) -> Self {
        Self {
            shape: array.shape(),
            ndims: array.ndims(),
            data: array.data().as_ptr(),
        }
    }

    pub fn ndims(&self) -> isize {
        self.ndims
    }

    pub fn len(&self) -> isize {
        self.shape.dim(self.ndims() - 1)
    }

    fn stride(&self) -> isize {
        self.shape
            .iter()
            .take((self.ndims() - 1) as usize)
            .product()
    }

    pub fn at(&'a self, index: isize) -> Subarray<'a, T> {
        if 0 <= index && index < self.len() {
            if self.ndims() > 0 {
                Self {
                    shape: self.shape,
                    ndims: self.ndims() - 1,
                    data: unsafe { self.data.offset(index * self.stride()) },
                }
            } else {
                *self
            }
        } else {
            panic!("index is {}, but len is {}", index, self.len())
        }
    }

    pub fn read(&self) -> T {
        unsafe { *self.data }
    }
}