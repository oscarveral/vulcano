pub trait OverflowingMul<Rhs = Self> {
    type Output;

    fn overflowing_mul(self, rhs: Rhs) -> (Self::Output, bool);
}

macro_rules! impl_overflowing_mul {
    ($($t: ty),+) => {$(
        impl OverflowingMul<$t> for $t {
            type Output = $t;

            #[inline]
            fn overflowing_mul(self, rhs: $t) -> (Self::Output, bool) {
                <$t>::overflowing_mul(self, rhs)
            }
        }
    )+};
}

impl_overflowing_mul!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
