use std::fmt::Debug;

// impl'd for types that know what type they are
pub trait TypeAware: Copy + PartialEq + Debug {
    const ID: u8;

    // this can be removed
    const BYTES: usize;

    const LABEL: &'static str;
}
