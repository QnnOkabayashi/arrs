use std::fmt;
use std::iter::Iterator;

mod tests {
    use super::*;

    #[test]
    fn non_converging() {
        let result = Range::new(10, 0, Some(1));
        assert_eq!(
            result,
            Err(RangeError::NonConverging(RawRange {
                start: 10,
                stop: 0,
                step: 1,
            }))
        )
    }

    #[test]
    fn zero_step() {
        let result = Range::new(1, 10, Some(0));
        assert_eq!(result, Err(RangeError::ZeroStep))
    }

    #[test]
    fn range_len() {
        assert_eq!(0, Range::new(0, 0, Some(1)).unwrap().len());
        assert_eq!(0, Range::new(0, 0, Some(-20)).unwrap().len());
        assert_eq!(5, Range::new(0, 9, Some(2)).unwrap().len());
        assert_eq!(4, Range::new(0, 7, Some(2)).unwrap().len());
        assert_eq!(3, Range::new(0, 7, Some(3)).unwrap().len());
        assert_eq!(8, Range::new(0, 8, None).unwrap().len());
        assert_eq!(1, Range::new(9, 8, None).unwrap().len());
    }
}

#[derive(Debug, PartialEq, Clone)]
struct RawRange {
    start: isize,
    stop: isize,
    step: isize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Range {
    raw: RawRange,
    len: usize,
}

impl Range {
    fn validated_new(start: isize, stop: isize, step: isize) -> Range {
        let raw = RawRange { start, stop, step };

        // calculate length of range
        // uses 2 conditionals rn :(
        let dist = (stop - start).abs();
        let len = if dist != 0 {
            ((dist - 1) / step) + 1
        } else {
            0
        } as usize;

        Range { raw, len }
    }

    pub fn new(start: isize, stop: isize, step: Option<isize>) -> RangeResult {
        if let Some(step) = step {
            if step == 0 {
                Err(RangeError::ZeroStep)
            } else if start == stop || (start < stop) == (step > 0) {
                Ok(Self::validated_new(start, stop, step))
            } else {
                Err(RangeError::NonConverging(RawRange { start, stop, step }))
            }
        } else {
            let step = if start > stop { -1 } else { 1 };
            Ok(Self::validated_new(start, stop, step))
        }
    }

    pub fn empty() -> Self {
        let (start, stop, step) = (0, 0, 1);
        Self::validated_new(start, stop, step)
    }

    pub fn bounds(&self) -> (isize, isize, isize) {
        (self.raw.start, self.raw.stop, self.raw.step)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> RangeIterator {
        RangeIterator {
            range: self,
            index: self.raw.start,
            pos: 0,
        }
    }
}

pub struct RangeIterator<'a> {
    range: &'a Range,
    index: isize,
    pos: usize,
}

impl Iterator for RangeIterator<'_> {
    type Item = isize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 1 < self.range.len() {
            self.pos += 1;
            self.index += self.range.raw.step;
            Some(self.index)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RangeError {
    NonConverging(RawRange),
    ZeroStep,
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RangeError::NonConverging(RawRange { start, stop, step }) => {
                let step_sign = if step > 0 { "positive" } else { "negative" };

                write!(
                    f,
                    "range doesn't converge with start: {}, and stop: {}, when step is {}",
                    start, stop, step_sign,
                )
            }
            RangeError::ZeroStep => {
                write!(f, "range step cannot be zero")
            }
        }
    }
}

pub type RangeResult = std::result::Result<Range, RangeError>;
