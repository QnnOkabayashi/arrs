use super::dtype::TypeConscious;

// only make one cursor and pass it around between ArrayDimIterator's
// each one is constructed whenever you need to do stuff with the data
pub struct ArrayCursor<'a, T>
where
    T: TypeConscious
{
    inner: &'a Vec<T>,
    pos: usize,
}

impl<'a, T> ArrayCursor<'a, T>
where
    T: TypeConscious,
{
    pub fn new(inner: &'a Vec<T>) -> Self {
        Self { inner, pos: 0 }
    }

    pub fn move_pos(&mut self, len: usize) {
        // TODO: add error checking here
        self.pos += len
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn read(&self) -> &T {
        &self.inner[self.pos]
    }
}