use std::fmt::Display;

macro_rules! impl_integer_from_bytes {
    ($($type:ty),*) => {
        $(
            impl Integer for $type {
                fn new() -> Self {
                    0
                }

                fn from_ne_bytes(bytes: &[u8]) -> Self {
                    if bytes.len() != std::mem::size_of::<$type>() {
                        panic!("Invalid byte length for type");
                    }
                    let array: [u8; std::mem::size_of::<$type>()] = bytes.try_into().expect("Failed to convert bytes to array");
                    <$type>::from_ne_bytes(array)
                }

                fn to_ne_bytes(self) -> Vec<u8> {
                    self.to_ne_bytes().to_vec()
                }
            }
        )*
    };
}

pub trait Integer: Sized + Copy + PartialEq + PartialOrd + Display {
    fn new() -> Self;
    fn from_ne_bytes(bytes: &[u8]) -> Self;
    fn to_ne_bytes(self) -> Vec<u8>;
}

// Implement the trait for various integer types using the macro
impl_integer_from_bytes!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
