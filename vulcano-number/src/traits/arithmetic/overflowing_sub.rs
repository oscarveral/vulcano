pub trait OverflowingSub<Rhs = Self> {
    type Output;

    fn overflowing_sub(self, rhs: Rhs) -> (Self::Output, bool);
}

macro_rules! impl_overflowing_sub {
    ($($t: ty),+) => {$(
        impl OverflowingSub<$t> for $t {
            type Output = $t;

            #[inline]
            fn overflowing_sub(self, rhs: $t) -> (Self::Output, bool) {
                <$t>::overflowing_sub(self, rhs)
            }
        }
    )+};
}

impl_overflowing_sub!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
