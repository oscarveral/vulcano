pub trait CheckedAdd<Rhs = Self> {
    type Output;

    fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}

macro_rules! impl_checked_add {
    ($($t: ty),+) => {$(
        impl CheckedAdd<$t> for $t {
            type Output = $t;

            #[inline]
            fn checked_add(self, rhs: $t) -> Option<Self::Output> {
                <$t>::checked_add(self, rhs)
            }
        }
    )+};
}

impl_checked_add!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
