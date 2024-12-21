pub trait CheckedSub<Rhs = Self> {
    type Output;

    fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}

pub trait OverflowingSub<Rhs = Self> {
    type Output;

    fn overflowing_sub(self, rhs: Rhs) -> (Self::Output, bool);
}

pub trait SaturatingSub<Rhs = Self> {
    type Output;

    fn saturating_sub(self, rhs: Rhs) -> Self::Output;
}

pub trait WrappingSub<Rhs = Self> {
    type Output;

    fn wrapping_sub(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_sub {
    ($($t: ty),+) => {$(
        impl CheckedSub<$t> for $t {
            type Output = $t;

            #[inline]
            fn checked_sub(self, rhs: $t) -> Option<Self::Output> {
                <$t>::checked_sub(self, rhs)
            }
        }
        impl OverflowingSub<$t> for $t {
            type Output = $t;

            #[inline]
            fn overflowing_sub(self, rhs: $t) -> (Self::Output, bool) {
                <$t>::overflowing_sub(self, rhs)
            }
        }
        impl SaturatingSub<$t> for $t {
            type Output = $t;

            #[inline]
            fn saturating_sub(self, rhs: $t) -> Self::Output {
                <$t>::saturating_sub(self, rhs)
            }
        }
        impl WrappingSub<$t> for $t {
            type Output = $t;

            #[inline]
            fn wrapping_sub(self, rhs: $t) -> Self::Output {
                <$t>::wrapping_sub(self, rhs)
            }
        }
    )+};
}

impl_sub!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
