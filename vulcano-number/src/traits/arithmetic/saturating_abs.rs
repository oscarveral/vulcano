pub trait SaturatingAbs {
    type Output;

    fn saturating_abs(self) -> Self::Output;
}

macro_rules! impl_saturating_abs {
    ($($t: ty),+) => {$(
        impl SaturatingAbs for $t {
            type Output = $t;

            #[inline]
            fn saturating_abs(self) -> Self::Output {
                <$t>::saturating_abs(self)
            }
        }
    )+};
}

impl_saturating_abs!(i8, i16, i32, i64, i128, isize);
