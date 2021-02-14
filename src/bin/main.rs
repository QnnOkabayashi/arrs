use arrs::{array::{Array, Shape}, serde_arrs};
use std::fs;


fn main() {
    let bytes = fs::read("idx-files/t10k-images-idx3-ubyte").unwrap();

    let test_imgs = serde_arrs::from_bytes::<u8>(bytes).unwrap();

    println!("test_imgs shape: {}", test_imgs.shape())
}