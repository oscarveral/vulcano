pub trait CheckedMul<Rhs = Self> {
    type Output;

    fn checked_mul(self, rhs: Rhs) -> Option<Self::Output>;
}

macro_rules! impl_checked_mul {
    ($($t: ty),+) => {$(
        impl CheckedMul<$t> for $t {
            type Output = $t;

            #[inline]
            fn checked_mul(self, rhs: Self) -> Option<Self::Output> {
                <$t>::checked_mul(self, rhs)
            }
        }
    )+};
}

impl_checked_mul!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
