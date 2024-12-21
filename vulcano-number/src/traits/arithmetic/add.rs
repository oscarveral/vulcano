pub trait CheckedAdd<Rhs = Self> {
    type Output;

    fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait OverflowingAdd<Rhs = Self> {
    type Output;

    fn overflowing_add(self, rhs: Rhs) -> (Self::Output, bool);
}

pub trait SaturatingAdd<Rhs = Self> {
    type Output;

    fn saturating_add(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingAdd<Rhs = Self> {
    type Output;

    fn wrapping_add(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_add {
    ($($t: ty),+) => {$(
        impl CheckedAdd<$t> for $t {
            type Output = $t;

            #[inline]
            fn checked_add(self, rhs: $t) -> Option<Self::Output> {
                <$t>::checked_add(self, rhs)
            }
        }
        impl OverflowingAdd<$t> for $t {
            type Output = $t;

            #[inline]
            fn overflowing_add(self, rhs: $t) -> (Self::Output, bool) {
                <$t>::overflowing_add(self, rhs)
            }
        }
        impl SaturatingAdd<$t> for $t {
            type Output = $t;

            #[inline]
            fn saturating_add(self, rhs: $t) -> Self::Output {
                <$t>::saturating_add(self, rhs)
            }
        }
        impl WrappingAdd<$t> for $t {
            type Output = $t;

            #[inline]
            fn wrapping_add(self, rhs: $t) -> Self::Output {
                <$t>::wrapping_add(self, rhs)
            }
        }
    )+};
}

impl_add!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
