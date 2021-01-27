use super::dtype::TypeAware;

// A basic implementation of an immutable cursor.
// Improve later; this is just for testing purposes
// and can probably be improved by using the raw pointer
// but that's not a priority right now.
// probably using Vec::as_ptr()
pub struct ArrayCursor<'a, T>
where
    T: TypeAware,
{
    inner: &'a Vec<T>,
    pos: isize,
}

impl<'a, T> ArrayCursor<'a, T>
where
    T: TypeAware,
{
    pub fn new(inner: &'a Vec<T>) -> Self {
        Self { inner, pos: 0 }
    }

    pub fn get_pos(&self) -> isize {
        self.pos
    }

    pub fn set_pos(&mut self, pos: isize) {
        self.pos = pos
    }

    pub fn move_pos(&mut self, offset: isize) {
        self.set_pos(self.pos + offset)
    }

    // panics if the index is out of bounds
    pub fn read(&self) -> T {
        // TODO: figure out OutOfBounds indexing
        // performs a copy here
        self.inner[self.pos as usize]
    }
}
