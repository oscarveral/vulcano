pub trait WrappingNeg {
    type Output;

    fn wrapping_neg(self) -> Self::Output;
}

macro_rules! impl_wrapping_neg {
    ($($t: ty),+) => {$(
        impl WrappingNeg for $t {
            type Output = $t;

            #[inline]
            fn wrapping_neg(self) -> Self::Output {
                <$t>::wrapping_neg(self)
            }
        }
    )+};
}

impl_wrapping_neg!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
