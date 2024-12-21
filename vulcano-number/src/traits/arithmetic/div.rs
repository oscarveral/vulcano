pub trait CheckedDiv<Rhs = Self> {
    type Output;

    fn checked_div(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait OverflowingDiv<Rhs = Self> {
    type Output;

    fn overflowing_div(self, rhs: Rhs) -> (Self::Output, bool);
}

pub trait SaturatingDiv<Rhs = Self> {
    type Output;

    fn saturating_div(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingDiv<Rhs = Self> {
    type Output;

    fn wrapping_div(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_div {
    ($($t: ty),+) => {$(
        impl CheckedDiv<$t> for $t {
            type Output = $t;

            #[inline]
            fn checked_div(self, rhs: Self) -> Option<Self::Output> {
                <$t>::checked_div(self, rhs)
            }
        }
        impl OverflowingDiv<$t> for $t {
            type Output = $t;

            #[inline]
            fn overflowing_div(self, rhs: $t) -> (Self::Output, bool) {
                <$t>::overflowing_div(self, rhs)
            }
        }
        impl SaturatingDiv<$t> for $t {
            type Output = $t;

            #[inline]
            fn saturating_div(self, rhs: $t) -> Self::Output {
                <$t>::saturating_div(self, rhs)
            }
        }
        impl WrappingDiv<$t> for $t {
            type Output = $t;

            #[inline]
            fn wrapping_div(self, rhs: $t) -> Self::Output {
                <$t>::wrapping_div(self, rhs)
            }
        }
    )+};
}

impl_div!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
