pub trait WrappingAbs {
    type Output;

    fn wrapping_abs(self) -> Self::Output;
}

macro_rules! impl_wrapping_abs {
    ($($t: ty),+) => {$(
        impl WrappingAbs for $t {
            type Output = $t;

            #[inline]
            fn wrapping_abs(self) -> Self::Output {
                <$t>::wrapping_abs(self)
            }
        }
    )+};
}

impl_wrapping_abs!(i8, i16, i32, i64, i128, isize);
