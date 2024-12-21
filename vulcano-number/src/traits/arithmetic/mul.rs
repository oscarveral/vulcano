pub trait CheckedMul<Rhs = Self> {
    type Output;

    fn checked_mul(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait OverflowingMul<Rhs = Self> {
    type Output;

    fn overflowing_mul(self, rhs: Rhs) -> (Self::Output, bool);
}

pub trait SaturatingMul<Rhs = Self> {
    type Output;

    fn saturating_mul(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingMul<Rhs = Self> {
    type Output;

    fn wrapping_mul(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_mul {
    ($($t: ty),+) => {$(
        impl CheckedMul<$t> for $t {
            type Output = $t;

            #[inline]
            fn checked_mul(self, rhs: Self) -> Option<Self::Output> {
                <$t>::checked_mul(self, rhs)
            }
        }
        impl OverflowingMul<$t> for $t {
            type Output = $t;

            #[inline]
            fn overflowing_mul(self, rhs: $t) -> (Self::Output, bool) {
                <$t>::overflowing_mul(self, rhs)
            }
        }
        impl SaturatingMul<$t> for $t {
            type Output = $t;

            #[inline]
            fn saturating_mul(self, rhs: $t) -> Self::Output {
                <$t>::saturating_mul(self, rhs)
            }
        }
        impl WrappingMul<$t> for $t {
            type Output = $t;

            #[inline]
            fn wrapping_mul(self, rhs: $t) -> Self::Output {
                <$t>::wrapping_mul(self, rhs)
            }
        }
    )+};
}

impl_mul!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
