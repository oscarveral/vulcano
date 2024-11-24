pub trait Fixed {
    fn bit_width(&self) -> u32;
    fn max_value(&self) -> Self;
    fn min_value(&self) -> Self;
}

macro_rules! impl_fixed {
    ($($t: ty),+) => {$(
        impl Fixed for $t {
            #[inline]
            fn bit_width(&self) -> u32 { <$t>::BITS }
            #[inline]
            fn max_value(&self) -> Self { <$t>::MAX }
            #[inline]
            fn min_value(&self) -> Self { <$t>::MIN }
        }
    )+};
}

impl_fixed!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
