pub trait OverflowingAbs {
    type Output;

    fn overflowing_abs(self) -> (Self::Output, bool);
}

macro_rules! impl_overflowing_abs {
    ($($t: ty),+) => {$(
        impl OverflowingAbs for $t {
            type Output = $t;

            #[inline]
            fn overflowing_abs(self) -> (Self::Output, bool) {
                <$t>::overflowing_abs(self)
            }
        }
    )+};
}

impl_overflowing_abs!(i8, i16, i32, i64, i128, isize);
