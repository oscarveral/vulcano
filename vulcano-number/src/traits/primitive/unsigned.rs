use crate::traits::primitive::Integer;

pub trait Unsigned: Integer {}

macro_rules! impl_unsigned {
    ($($t: ty),+) => {$(
        impl Unsigned for $t {}
    )+};
}

impl_unsigned!(u8, u16, u32, u64, u128, usize);
