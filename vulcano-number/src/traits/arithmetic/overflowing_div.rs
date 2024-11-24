pub trait OverflowingDiv<Rhs = Self> {
    type Output;

    fn overflowing_div(self, rhs: Rhs) -> (Self::Output, bool);
}

macro_rules! impl_overflowing_div {
    ($($t: ty),+) => {$(
        impl OverflowingDiv<$t> for $t {
            type Output = $t;

            #[inline]
            fn overflowing_div(self, rhs: $t) -> (Self::Output, bool) {
                <$t>::overflowing_div(self, rhs)
            }
        }
    )+};
}

impl_overflowing_div!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
