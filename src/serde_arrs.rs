mod de;
mod error;
mod ser;
use super::array::{Array, DType, Shape, TypeAware};
use de::BinDeserializer;
use error::{Error, Result, ResultV};
use ser::Serializer;

trait Endianess {
    fn from_be_bytes(bytes: &[u8]) -> Self;

    fn from_le_bytes(bytes: &[u8]) -> Self;

    fn to_be_bytes(&self) -> [u8];

    fn to_le_bytes(self) -> [u8];
}

#[test]
fn compile() {}
