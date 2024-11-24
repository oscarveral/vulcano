use core::ops::Neg;

use crate::traits::{
    arithmetic::{CheckedAbs, OverflowingAbs, SaturatingAbs, WrappingAbs},
    primitive::Integer,
};

pub trait Signed:
    Integer
    + Neg<Output = Self>
    + CheckedAbs<Output = Self>
    + OverflowingAbs<Output = Self>
    + SaturatingAbs<Output = Self>
    + WrappingAbs<Output = Self>
{
}

macro_rules! impl_signed {
    ($($t: ty),+) => {$(
        impl Signed for $t {}
    )+};
}

impl_signed!(i8, i16, i32, i64, i128, isize);
