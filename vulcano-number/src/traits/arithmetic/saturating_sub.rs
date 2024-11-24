pub trait SaturatingSub<Rhs = Self> {
    type Output;

    fn saturating_sub(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_saturating_sub {
    ($($t: ty),+) => {$(
        impl SaturatingSub<$t> for $t {
            type Output = $t;

            #[inline]
            fn saturating_sub(self, rhs: $t) -> Self::Output {
                <$t>::saturating_sub(self, rhs)
            }
        }
    )+};
}

impl_saturating_sub!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
