pub trait OverflowingAdd<Rhs = Self> {
    type Output;

    fn overflowing_add(self, rhs: Rhs) -> (Self::Output, bool);
}

macro_rules! impl_overflowing_add {
    ($($t: ty),+) => {$(
        impl OverflowingAdd<$t> for $t {
            type Output = $t;

            #[inline]
            fn overflowing_add(self, rhs: $t) -> (Self::Output, bool) {
                <$t>::overflowing_add(self, rhs)
            }
        }
    )+};
}

impl_overflowing_add!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
