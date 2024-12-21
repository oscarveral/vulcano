use core::{
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
        DivAssign, Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign,
    },
    str::FromStr,
};

use crate::traits::{
    arithmetic::{
        CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedSub, OverflowingAdd, OverflowingDiv,
        OverflowingMul, OverflowingNeg, OverflowingSub, SaturatingAdd, SaturatingDiv,
        SaturatingMul, SaturatingSub, WrappingAdd, WrappingDiv, WrappingMul, WrappingNeg,
        WrappingSub,
    },
    conversion::Convertible,
    size::Fixed,
};

pub trait Integer:
    Add<Self, Output = Self>
    + AddAssign<Self>
    + BitAnd<Self, Output = Self>
    + BitAndAssign<Self>
    + BitOr<Self, Output = Self>
    + BitOrAssign<Self>
    + BitXor<Self, Output = Self>
    + BitXorAssign<Self>
    + CheckedAdd<Self, Output = Self>
    + CheckedDiv<Self, Output = Self>
    + CheckedMul<Self, Output = Self>
    + CheckedNeg<Output = Self>
    + CheckedSub<Self, Output = Self>
    + Clone
    + Convertible<Self>
    + Convertible<u8>
    + Convertible<u16>
    + Convertible<u32>
    + Convertible<u64>
    + Convertible<u128>
    + Convertible<usize>
    + Convertible<i8>
    + Convertible<i16>
    + Convertible<i32>
    + Convertible<i64>
    + Convertible<i128>
    + Convertible<isize>
    + Copy
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + Eq
    + Fixed
    + FromStr
    + Mul<Self, Output = Self>
    + MulAssign<Self>
    + Not<Output = Self>
    + Ord
    + OverflowingAdd<Self, Output = Self>
    + OverflowingDiv<Self, Output = Self>
    + OverflowingMul<Self, Output = Self>
    + OverflowingNeg<Output = Self>
    + OverflowingSub<Self, Output = Self>
    + PartialEq
    + PartialOrd
    + Rem<Self, Output = Self>
    + RemAssign<Self>
    + SaturatingAdd<Self, Output = Self>
    + SaturatingDiv<Self, Output = Self>
    + SaturatingMul<Self, Output = Self>
    + SaturatingSub<Self, Output = Self>
    + Shl<Self, Output = Self>
    + ShlAssign<Self>
    + Shr<Self, Output = Self>
    + ShrAssign<Self>
    + Sized
    + WrappingAdd<Self, Output = Self>
    + WrappingDiv<Self, Output = Self>
    + WrappingMul<Self, Output = Self>
    + WrappingNeg<Output = Self>
    + WrappingSub<Self, Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
}

macro_rules! impl_integer {
    ($($t: ty),+) => {$(
        impl Integer for $t {
            #[inline]
            fn zero() -> Self { 0 }
            #[inline]
            fn one() -> Self { 1 }
        }
    )+};
}

impl_integer!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
