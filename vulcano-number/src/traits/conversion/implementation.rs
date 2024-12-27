use crate::traits::conversion::{OverflowingFrom, SaturatingFrom, WrappingFrom};

macro_rules! impl_identity_conversion {
    ($($t: ty),+) => {$(
        impl OverflowingFrom<$t> for $t {
            #[inline]
            fn overflowing_from(value: $t) -> ($t, bool) {
                (value, false)
            }
        }
        impl SaturatingFrom<$t> for $t {
            #[inline]
            fn saturating_from(value: $t) -> $t {
                value
            }
        }
        impl WrappingFrom<$t> for $t {
            #[inline]
            fn wrapping_from(value: $t) -> $t {
                value
            }
        }
    )+};
}

macro_rules! impl_lossless_conversion {
    ($small: ty, $large: ty) => {
        impl OverflowingFrom<$small> for $large {
            #[inline]
            fn overflowing_from(value: $small) -> ($large, bool) {
                (<$large>::from(value), false)
            }
        }
        impl SaturatingFrom<$small> for $large {
            #[inline]
            fn saturating_from(value: $small) -> $large {
                <$large>::from(value)
            }
        }
        impl WrappingFrom<$small> for $large {
            #[inline]
            fn wrapping_from(value: $small) -> $large {
                <$large>::from(value)
            }
        }
    };
}

macro_rules! impl_lossy_conversion {
    ($a: ty, $b: ty) => {
        impl OverflowingFrom<$a> for $b {
            #[inline]
            fn overflowing_from(value: $a) -> ($b, bool) {
                let res = <$b>::wrapping_from(value);
                (res, <$a>::wrapping_from(res) != value)
            }
        }
        impl SaturatingFrom<$a> for $b {
            #[inline]
            fn saturating_from(value: $a) -> $b {
                if let Ok(max) = <$a>::try_from(<$b>::MAX) {
                    if value >= max {
                        return <$b>::MAX;
                    }
                }
                if let Ok(min) = <$a>::try_from(<$b>::MIN) {
                    if value <= min {
                        return <$b>::MIN;
                    }
                }
                <$b>::wrapping_from(value)
            }
        }
        impl WrappingFrom<$a> for $b {
            #[inline]
            fn wrapping_from(value: $a) -> $b {
                value as $b
            }
        }
    };
}

macro_rules! impl_contained_conversion {
    ($a: ty, $b: ty) => {
        impl_lossless_conversion!($a, $b);
        impl_lossy_conversion!($b, $a);
    };
}

macro_rules! impl_no_contained_conversion {
    ($a: ty, $b: ty) => {
        impl_lossy_conversion!($a, $b);
        impl_lossy_conversion!($b, $a);
    };
}

macro_rules! impl_lossless_conversion_pointer {
    ($small: ty, $large: ty) => {
        impl OverflowingFrom<$small> for $large {
            #[inline]
            fn overflowing_from(value: $small) -> ($large, bool) {
                (value as $large, false)
            }
        }
        impl SaturatingFrom<$small> for $large {
            #[inline]
            fn saturating_from(value: $small) -> $large {
                value as $large
            }
        }
        impl WrappingFrom<$small> for $large {
            #[inline]
            fn wrapping_from(value: $small) -> $large {
                <$large>::try_from(value).ok().unwrap()
            }
        }
    };
}

macro_rules! impl_contained_conversion_pointer {
    ($a: ty, $b: ty) => {
        impl_lossless_conversion_pointer!($a, $b);
        impl_lossy_conversion!($b, $a);
    };
}

impl_identity_conversion!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl_contained_conversion!(u8, u16);
impl_contained_conversion!(u8, u32);
impl_contained_conversion!(u8, u64);
impl_contained_conversion!(u8, u128);
impl_contained_conversion!(u8, i16);
impl_contained_conversion!(u8, i32);
impl_contained_conversion!(u8, i64);
impl_contained_conversion!(u8, i128);

impl_contained_conversion!(u16, u32);
impl_contained_conversion!(u16, u64);
impl_contained_conversion!(u16, u128);
impl_contained_conversion!(u16, i32);
impl_contained_conversion!(u16, i64);
impl_contained_conversion!(u16, i128);

impl_contained_conversion!(u32, u64);
impl_contained_conversion!(u32, u128);
impl_contained_conversion!(u32, i64);
impl_contained_conversion!(u32, i128);

impl_contained_conversion!(u64, u128);
impl_contained_conversion!(u64, i128);

impl_contained_conversion!(i8, i16);
impl_contained_conversion!(i8, i32);
impl_contained_conversion!(i8, i64);
impl_contained_conversion!(i8, i128);
impl_contained_conversion!(i16, i32);
impl_contained_conversion!(i16, i64);
impl_contained_conversion!(i16, i128);
impl_contained_conversion!(i32, i64);
impl_contained_conversion!(i32, i128);
impl_contained_conversion!(i64, i128);

impl_no_contained_conversion!(u8, i8);
impl_no_contained_conversion!(u16, i8);
impl_no_contained_conversion!(u16, i16);
impl_no_contained_conversion!(u32, i8);
impl_no_contained_conversion!(u32, i16);
impl_no_contained_conversion!(u32, i32);
impl_no_contained_conversion!(u64, i8);
impl_no_contained_conversion!(u64, i16);
impl_no_contained_conversion!(u64, i32);
impl_no_contained_conversion!(u64, i64);
impl_no_contained_conversion!(u128, i8);
impl_no_contained_conversion!(u128, i16);
impl_no_contained_conversion!(u128, i32);
impl_no_contained_conversion!(u128, i64);
impl_no_contained_conversion!(u128, i128);

#[cfg(target_pointer_width = "16")]
mod pointer {
    use super::*;

    impl_contained_conversion!(u8, usize);
    impl_contained_conversion!(u16, usize);
    impl_no_contained_conversion!(u32, usize);
    impl_no_contained_conversion!(u64, usize);
    impl_no_contained_conversion!(u128, usize);

    impl_contained_conversion!(i8, usize);
    impl_no_contained_conversion!(i16, usize);
    impl_no_contained_conversion!(i32, usize);
    impl_no_contained_conversion!(i64, usize);
    impl_no_contained_conversion!(i128, usize);

    impl_contained_conversion!(u8, isize);
    impl_no_contained_conversion!(u16, isize);
    impl_no_contained_conversion!(u32, isize);
    impl_no_contained_conversion!(u64, isize);
    impl_no_contained_conversion!(u128, isize);

    impl_contained_conversion!(i8, isize);
    impl_contained_conversion!(i16, isize);
    impl_no_contained_conversion!(i32, isize);
    impl_no_contained_conversion!(i64, isize);
    impl_no_contained_conversion!(i128, isize);

    impl_no_contained_conversion!(usize, isize);
}

#[cfg(target_pointer_width = "32")]
mod pointer {
    use super::*;

    impl_contained_conversion!(u8, usize);
    impl_contained_conversion!(u16, usize);
    impl_contained_conversion!(u32, usize);
    impl_no_contained_conversion!(u64, usize);
    impl_no_contained_conversion!(u128, usize);

    impl_contained_conversion!(i8, usize);
    impl_contained_conversion!(i16, usize);
    impl_no_contained_conversion!(i32, usize);
    impl_no_contained_conversion!(i64, usize);
    impl_no_contained_conversion!(i128, usize);

    impl_contained_conversion!(u8, isize);
    impl_contained_conversion!(u16, isize);
    impl_no_contained_conversion!(u32, isize);
    impl_no_contained_conversion!(u64, isize);
    impl_no_contained_conversion!(u128, isize);

    impl_contained_conversion!(i8, isize);
    impl_contained_conversion!(i16, isize);
    impl_contained_conversion!(i32, isize);
    impl_no_contained_conversion!(i64, isize);
    impl_no_contained_conversion!(i128, isize);

    impl_no_contained_conversion!(usize, isize);
}

#[cfg(target_pointer_width = "64")]
mod pointer {

    use super::*;

    impl_contained_conversion_pointer!(u8, usize);
    impl_contained_conversion_pointer!(u16, usize);
    impl_contained_conversion_pointer!(u32, usize);
    impl_contained_conversion_pointer!(u64, usize);
    impl_no_contained_conversion!(u128, usize);

    impl_contained_conversion_pointer!(i8, usize);
    impl_contained_conversion_pointer!(i16, usize);
    impl_contained_conversion_pointer!(i32, usize);
    impl_no_contained_conversion!(i64, usize);
    impl_no_contained_conversion!(i128, usize);

    impl_contained_conversion_pointer!(u8, isize);
    impl_contained_conversion_pointer!(u16, isize);
    impl_contained_conversion_pointer!(u32, isize);
    impl_no_contained_conversion!(u64, isize);
    impl_no_contained_conversion!(u128, isize);

    impl_contained_conversion_pointer!(i8, isize);
    impl_contained_conversion_pointer!(i16, isize);
    impl_contained_conversion_pointer!(i32, isize);
    impl_contained_conversion_pointer!(i64, isize);
    impl_no_contained_conversion!(i128, isize);

    impl_no_contained_conversion!(usize, isize);
}
