pub trait SaturatingMul<Rhs = Self> {
    type Output;

    fn saturating_mul(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_saturating_mul {
    ($($t: ty),+) => {$(
        impl SaturatingMul<$t> for $t {
            type Output = $t;

            #[inline]
            fn saturating_mul(self, rhs: $t) -> Self::Output {
                <$t>::saturating_mul(self, rhs)
            }
        }
    )+};
}

impl_saturating_mul!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
