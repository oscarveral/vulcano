use crate::traits::conversion::{
    ExactFrom, ExactInto, OverflowingFrom, OverflowingInto, SaturatingFrom, SaturatingInto,
    WrappingFrom, WrappingInto,
};

pub trait Convertible<T>:
    ExactFrom<T>
    + WrappingFrom<T>
    + OverflowingFrom<T>
    + SaturatingFrom<T>
    + ExactInto<T>
    + WrappingInto<T>
    + OverflowingInto<T>
    + SaturatingInto<T>
{
}

impl<T, U> Convertible<U> for T where
    T: ExactFrom<U>
        + WrappingFrom<U>
        + OverflowingFrom<U>
        + SaturatingFrom<U>
        + ExactInto<U>
        + WrappingInto<U>
        + OverflowingInto<U>
        + SaturatingInto<U>
{
}
