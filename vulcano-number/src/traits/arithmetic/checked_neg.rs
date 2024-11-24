pub trait CheckedNeg {
    type Output;

    fn checked_neg(self) -> Option<Self::Output>;
}

macro_rules! impl_checked_neg {
    ($($t: ty),+) => {$(
        impl CheckedNeg for $t {
            type Output = $t;

            #[inline]
            fn checked_neg(self) -> Option<Self::Output> {
                <$t>::checked_neg(self)
            }
        }
    )+};
}

impl_checked_neg!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
