pub trait CheckedAbs {
    type Output;

    fn checked_abs(self) -> Option<Self::Output>;
}

pub trait OverflowingAbs {
    type Output;

    fn overflowing_abs(self) -> (Self::Output, bool);
}

pub trait SaturatingAbs {
    type Output;

    fn saturating_abs(self) -> Self::Output;
}

pub trait WrappingAbs {
    type Output;

    fn wrapping_abs(self) -> Self::Output;
}

macro_rules! impl_abs {
    ($($t: ty),+) => {$(
        impl CheckedAbs for $t {
            type Output = $t;

            #[inline]
            fn checked_abs(self) -> Option<Self::Output> {
                <$t>::checked_abs(self)
            }
        }
        impl OverflowingAbs for $t {
            type Output = $t;

            #[inline]
            fn overflowing_abs(self) -> (Self::Output, bool) {
                <$t>::overflowing_abs(self)
            }
        }
        impl SaturatingAbs for $t {
            type Output = $t;

            #[inline]
            fn saturating_abs(self) -> Self::Output {
                <$t>::saturating_abs(self)
            }
        }
        impl WrappingAbs for $t {
            type Output = $t;

            #[inline]
            fn wrapping_abs(self) -> Self::Output {
                <$t>::wrapping_abs(self)
            }
        }
    )+};
}

impl_abs!(i8, i16, i32, i64, i128, isize);
