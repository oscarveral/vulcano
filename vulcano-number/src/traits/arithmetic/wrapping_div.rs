pub trait WrappingDiv<Rhs = Self> {
    type Output;

    fn wrapping_div(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_wrapping_div {
    ($($t: ty),+) => {$(
        impl WrappingDiv<$t> for $t {
            type Output = $t;

            #[inline]
            fn wrapping_div(self, rhs: $t) -> Self::Output {
                <$t>::wrapping_div(self, rhs)
            }
        }
    )+};
}

impl_wrapping_div!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
