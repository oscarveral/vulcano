pub trait CheckedNeg {
    type Output;

    fn checked_neg(self) -> Option<Self::Output>;
}

pub trait OverflowingNeg {
    type Output;

    fn overflowing_neg(self) -> (Self::Output, bool);
}

pub trait SaturatingNeg {
    type Output;

    fn saturating_neg(self) -> Self::Output;
}

pub trait WrappingNeg {
    type Output;

    fn wrapping_neg(self) -> Self::Output;
}

macro_rules! impl_neg {
    ($($t: ty),+) => {$(
        impl CheckedNeg for $t {
            type Output = $t;

            #[inline]
            fn checked_neg(self) -> Option<Self::Output> {
                <$t>::checked_neg(self)
            }
        }
        impl OverflowingNeg for $t {
            type Output = $t;

            #[inline]
            fn overflowing_neg(self) -> (Self::Output, bool) {
                <$t>::overflowing_neg(self)
            }
        }
        impl SaturatingNeg for $t {
            type Output = $t;

            #[inline]
            fn saturating_neg(self) -> Self::Output {
                <$t>::saturating_sub(0, self)
            }
        }
        impl WrappingNeg for $t {
            type Output = $t;

            #[inline]
            fn wrapping_neg(self) -> Self::Output {
                <$t>::wrapping_neg(self)
            }
        }
    )+};
}

impl_neg!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
