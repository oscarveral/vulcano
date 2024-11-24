pub trait CheckedDiv<Rhs = Self> {
    type Output;

    fn checked_div(self, rhs: Rhs) -> Option<Self::Output>;
}

macro_rules! impl_checked_div {
    ($($t: ty),+) => {$(
        impl CheckedDiv<$t> for $t {
            type Output = $t;

            #[inline]
            fn checked_div(self, rhs: Self) -> Option<Self::Output> {
                <$t>::checked_div(self, rhs)
            }
        }
    )+};
}

impl_checked_div!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
