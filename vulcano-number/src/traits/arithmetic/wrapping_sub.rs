pub trait WrappingSub<Rhs = Self> {
    type Output;

    fn wrapping_sub(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_wrapping_sub {
    ($($t: ty),+) => {$(
        impl WrappingSub<$t> for $t {
            type Output = $t;

            #[inline]
            fn wrapping_sub(self, rhs: $t) -> Self::Output {
                <$t>::wrapping_sub(self, rhs)
            }
        }
    )+};
}

impl_wrapping_sub!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
