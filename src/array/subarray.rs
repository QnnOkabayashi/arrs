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
        let shape = array.shape();
        let ndims = array.ndims();
        let data = array.data_ptr();

        Self { shape, ndims, data }
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
        assert!(0 <= index && index < self.len(), "index is {}, but len is {}", index, self.len());
        if self.ndims() > 0 {
            let shape = self.shape;
            let ndims = self.ndims() - 1;

            // Safe because we guarenteed above that index won't be too large
            let data = unsafe { self.data.offset(index * self.stride()) };

            Self { shape, ndims, data }
        } else {
            *self
        }
    }

    pub fn read(&self) -> T {
        // Safe because self.data is never modified, and is safe at construction
        unsafe { *self.data }
    }
}
