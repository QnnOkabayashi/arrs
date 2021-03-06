use crate::array::{max_const, ArrResult, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// leftmost dim is innermost, rightmost dim is outermost
pub struct Shape<const NDIMS: usize>([usize; NDIMS]);

impl<const NDIMS: usize> Shape<NDIMS> {
    pub fn new(dims: [usize; NDIMS]) -> Self {
        Self(dims)
    }

    pub fn dims(&self) -> [usize; NDIMS] {
        self.0
    }

    pub fn to_vec(&self) -> Vec<usize> {
        self.0.to_vec()
    }

    pub fn volume(&self) -> usize {
        self.0.iter().product()
    }

    pub fn broadcast<const NDIMS2: usize>(
        &self,
        other: &Shape<NDIMS2>,
    ) -> ArrResult<(
        Shape<{ max_const(NDIMS, NDIMS2) }>,
        [BroadcastInstruction; max_const(NDIMS, NDIMS2)],
    )> {
        use BroadcastInstruction::*;
        let mut dims = [0; max_const(NDIMS, NDIMS2)];
        let mut instructions = [PushLinear; max_const(NDIMS, NDIMS2)];

        let (mut iter_a, mut iter_b) = (self.0.iter(), other.0.iter());
        let (mut stride_a, mut stride_b) = (1, 1);

        for (dim, instruction) in dims.iter_mut().zip(instructions.iter_mut()) {
            let (next_a, next_b) = (iter_a.next(), iter_b.next());

            let (d, i) = match (next_a, next_b) {
                (Some(&a), Some(&b)) if a == b => (a, RecurseLinear { stride_a, stride_b }),
                (Some(&a), Some(1)) | (Some(&a), None) => (a, RecurseStretchB { stride_a }),
                (Some(1), Some(&b)) | (None, Some(&b)) => (b, RecurseStretchA { stride_b }),
                (None, None) => unreachable!(),
                _ => {
                    return Err(Error::Broadcast {
                        dims1: self.0.to_vec(),
                        dims2: other.0.to_vec(),
                    })
                }
            };

            stride_a *= next_a.unwrap_or(&1);
            stride_b *= next_b.unwrap_or(&1);

            *dim = d;
            *instruction = i;
        }

        instructions[0] = match instructions[0] {
            RecurseLinear { .. } => PushLinear,
            RecurseStretchA { .. } => PushStretchA,
            RecurseStretchB { .. } => PushStretchB,
            _ => unreachable!(),
        };

        Ok((Shape(dims), instructions))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BroadcastInstruction {
    PushLinear,
    PushStretchA,
    PushStretchB,
    RecurseLinear { stride_a: usize, stride_b: usize },
    RecurseStretchA { stride_b: usize },
    RecurseStretchB { stride_a: usize },
}
