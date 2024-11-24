pub trait SaturatingAdd<Rhs = Self> {
    type Output;

    fn saturating_add(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_saturating_add {
    ($($t: ty),+) => {$(
        impl SaturatingAdd<$t> for $t {
            type Output = $t;

            #[inline]
            fn saturating_add(self, rhs: $t) -> Self::Output {
                <$t>::saturating_add(self, rhs)
            }
        }
    )+};
}

impl_saturating_add!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
