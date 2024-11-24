pub trait SaturatingDiv<Rhs = Self> {
    type Output;

    fn saturating_div(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_saturating_div {
    ($($t: ty),+) => {$(
        impl SaturatingDiv<$t> for $t {
            type Output = $t;

            #[inline]
            fn saturating_div(self, rhs: $t) -> Self::Output {
                <$t>::saturating_div(self, rhs)
            }
        }
    )+};
}

impl_saturating_div!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
