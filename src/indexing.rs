use std::rc::Rc;
mod range;
use range::Range;

trait PositionalIndex<T> {
    fn index(idx: isize) -> T;
}

mod tests {
    #[test]
    fn index_creation() {
        // let all = Traverser::new();
        // println!("The all indexer is: {:#?}", all);
        // let slice = all.slice(0, 10, None);
        // println!("The slice indexer is: {:#?}", slice);
        // let slice2 = slice.slice(0, 10, Some(2));
        // println!("The slice2 indexer is: {:#?}", slice2);
        // let indices = slice2.indices(vec![1, 4, 2]);
        // println!("The indices indexer is: {:#?}", indices);
    }
}

/// Used for indexing into an NDArray
#[derive(Debug, PartialEq, Clone)]
pub enum Indexer {
    Scalar(usize),
    Slice(Range),
    Indices(Vec<usize>),
}

pub struct IndexError {
    dim_length: usize,
    indexer: Indexer,
}

impl Indexer {
    fn scalar(k: usize) -> Self {
        Self::Scalar(k)
    }

    fn slice(start: isize, stop: isize, step: Option<isize>) -> Self {
        // TODO: add handling for errors like zero step
        Self::Slice(Range::new(start, stop, step).unwrap_or(Range::empty()))
    }

    fn indices(i: Vec<usize>) -> Self {
        Self::Indices(i)
    }

    fn len(&self) -> usize {
        match self {
            Self::Scalar(_) => 1,
            Self::Slice(range) => range.len(),
            Self::Indices(vec) => vec.len(),
        }
    }
}

// easiest to think about traversers as working on a massive 1D array

/// Used for traversing array elements
#[derive(Debug, PartialEq, Clone)]
pub struct Traverser {
    len: usize,
    indexer: Indexer,
    base: Option<Rc<Self>>,
}

impl Traverser {
    pub fn new(len: usize) -> Rc<Self> {
        Rc::new(Self {
            len,
            indexer: Indexer::slice(0, len as isize, None),
            base: None,
        })
    }

    pub fn empty() -> Rc<Self> {
        Rc::new(Self {
            len: 0,
            indexer: Indexer::slice(0, 0, None),
            base: None,
        })
    }

    pub fn slice(self: &Rc<Self>, indexer: Indexer) -> Rc<Self> {
        // TODO: add handling for errors like zero step

        let len = indexer.len();
        Rc::new(Self {
            len,
            indexer,
            base: Some(self.clone()),
        })
    }

    // pub fn indices(self: &Rc<Self>, indices: Vec<usize>) -> Rc<Self> {
    //     Rc::new(Self::Indices {
    //         indices,
    //         base: self.clone(),
    //     })
    // }

    // pub fn idx(&self, i: Indexer) -> Self {
    //     match *self {
    //         Self::All => self.clone(),
    //         Self::Empty => Self::Empty,
    //         _ => self.clone(),
    //     }
    // }

    // TODO: Create an out of bounds error type for indexing
    pub fn index(&self, i: isize) -> Option<isize> {
        None
        // match *self {
        //     Self::All => Some(i),
        //     Self::Empty => None,
        //     Self::Slice { range, base } => {
        //         let (start, stop, step) = range.bounds();
        //         // if step < 0, has to be greater than stop
        //         // if step > 0, has to be less than stop
        //         let stepped = start + (i * step);

        //         // if stepped == stop, then we overstepped
        //         // if stepped < stop and moving forward, then ok
        //         // if stepped > stop and moving backwards, then ok
        //         // where moving forwards = step > 0
        //         let in_range = (stepped != stop) && ((step > 0) == (stepped < stop));

        //         if in_range {
        //             base.index(stepped)
        //         } else {
        //             // TODO: return slice indexing error
        //             None
        //         }
        //     },
        //     Self::Indices { indices, base } => {
        //         None
        //         // if i < 0 {
        //         //     if -i > indices.len() as isize {

        //         //     }
        //         // }
        //         // if indices.len() < i {
        //         //     base.index(indices[i])
        //         // } else {
        //         //     None
        //         // }
        //     }
        // }
    }
}
