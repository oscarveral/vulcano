pub trait SaturatingNeg {
    type Output;

    fn saturating_neg(self) -> Self::Output;
}

macro_rules! impl_saturating_neg {
    ($($t: ty),+) => {$(
        impl SaturatingNeg for $t {
            type Output = $t;

            #[inline]
            fn saturating_neg(self) -> Self::Output {
                //self.saturating_neg()
                <$t>::saturating_neg(self)
            }
        }
    )+};
}

impl_saturating_neg!(i8, i16, i32, i64, i128, isize);
