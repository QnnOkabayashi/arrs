// impl'd for types that know what type they are
pub trait TypeAware: Copy + PartialEq {
    const ID: u8;

    const BYTES: usize;

    const LABEL: &'static str;
}
