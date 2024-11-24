pub trait CheckedSub<Rhs = Self> {
    type Output;

    fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}

macro_rules! impl_checked_sub {
    ($($t: ty),+) => {$(
        impl CheckedSub<$t> for $t {
            type Output = $t;

            #[inline]
            fn checked_sub(self, rhs: $t) -> Option<Self::Output> {
                <$t>::checked_sub(self, rhs)
            }
        }
    )+};
}

impl_checked_sub!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
