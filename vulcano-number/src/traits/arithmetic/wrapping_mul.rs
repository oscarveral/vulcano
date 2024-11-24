pub trait WrappingMul<Rhs = Self> {
    type Output;

    fn wrapping_mul(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_wrapping_mul {
    ($($t: ty),+) => {$(
        impl WrappingMul<$t> for $t {
            type Output = $t;

            #[inline]
            fn wrapping_mul(self, rhs: $t) -> Self::Output {
                <$t>::wrapping_mul(self, rhs)
            }
        }
    )+};
}

impl_wrapping_mul!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
