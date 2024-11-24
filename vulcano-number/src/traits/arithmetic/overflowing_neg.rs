pub trait OverflowingNeg {
    type Output;

    fn overflowing_neg(self) -> (Self::Output, bool);
}

macro_rules! impl_overflowing_neg {
    ($($t: ty),+) => {$(
        impl OverflowingNeg for $t {
            type Output = $t;

            #[inline]
            fn overflowing_neg(self) -> (Self::Output, bool) {
                <$t>::overflowing_neg(self)
            }
        }
    )+};
}

impl_overflowing_neg!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
