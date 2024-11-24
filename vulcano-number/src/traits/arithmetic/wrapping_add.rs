pub trait WrappingAdd<Rhs = Self> {
    type Output;

    fn wrapping_add(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_wrapping_add {
    ($($t: ty),+) => {$(
        impl WrappingAdd<$t> for $t {
            type Output = $t;

            #[inline]
            fn wrapping_add(self, rhs: $t) -> Self::Output {
                <$t>::wrapping_add(self, rhs)
            }
        }
    )+};
}

impl_wrapping_add!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
