use super::{Array1, Shape1, TypeAware};

#[derive(Clone, Copy)]
pub(super) struct Subarray<'a, T>
where
    T: TypeAware,
{
    shape: &'a Shape1,
    ndims: usize,
    data: *const T,
}

impl<'a, T> Subarray<'a, T>
where
    T: TypeAware,
{
    pub fn new(array: &'a Array1<T>) -> Self {
        Self {
            shape: array.shape(),
            ndims: array.ndims(),
            data: array.data_ptr(),
        }
    }

    pub fn ndims(&self) -> usize {
        self.ndims
    }

    pub fn len(&self) -> usize {
        self.shape.dim_at(self.ndims() - 1)
    }

    fn stride(&self) -> usize {
        // make this O(1) by calculating an accumulating stride vec at the start
        let v = self.shape.get_dims();
        v.iter().take((self.ndims() - 1) as usize).product()
    }

    pub fn at(&'a self, index: usize) -> Subarray<'a, T> {
        assert!(
            index < self.len(),
            "index is {}, but len is {}",
            index,
            self.len()
        );
        if self.ndims() > 0 {
            let shape = self.shape;
            let ndims = self.ndims() - 1;

            let offset = (index * self.stride()) as isize;
            // Safe because we guaranteed above that index won't be too large
            let data = unsafe { self.data.offset(offset) };

            Self { shape, ndims, data }
        } else {
            *self
        }
    }

    pub fn read(&self) -> T {
        // underlying data isn't modified while Subarray exists, pointer doesn't change after construction
        unsafe { *self.data }
    }
}
