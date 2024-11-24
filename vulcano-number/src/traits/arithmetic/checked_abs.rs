pub trait CheckedAbs {
    type Output;

    fn checked_abs(self) -> Option<Self::Output>;
}

macro_rules! impl_checked_abs {
    ($($t: ty),+) => {$(
        impl CheckedAbs for $t {
            type Output = $t;

            #[inline]
            fn checked_abs(self) -> Option<Self::Output> {
                <$t>::checked_abs(self)
            }
        }
    )+};
}

impl_checked_abs!(i8, i16, i32, i64, i128, isize);
