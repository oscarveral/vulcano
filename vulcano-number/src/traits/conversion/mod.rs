mod implementation;

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

pub trait ExactFrom<T> {
    fn exact_from(value: T) -> Self;
}

pub trait ExactInto<T> {
    fn exact_into(self) -> T;
}

impl<T, S: TryFrom<T>> ExactFrom<T> for S {
    fn exact_from(value: T) -> S {
        S::try_from(value).ok().unwrap()
    }
}

impl<T, S: ExactFrom<T>> ExactInto<S> for T {
    #[inline]
    fn exact_into(self) -> S {
        S::exact_from(self)
    }
}

pub trait OverflowingFrom<T>: Sized + WrappingFrom<T> {
    fn overflowing_from(value: T) -> (Self, bool);
}

pub trait OverflowingInto<T> {
    fn overflowing_into(self) -> (T, bool);
}

impl<T, S: OverflowingFrom<T>> OverflowingInto<S> for T {
    #[inline]
    fn overflowing_into(self) -> (S, bool) {
        S::overflowing_from(self)
    }
}

pub trait SaturatingFrom<T>: WrappingFrom<T> {
    fn saturating_from(value: T) -> Self;
}

pub trait SaturatingInto<T> {
    fn saturating_into(self) -> T;
}

impl<T, S: SaturatingFrom<T>> SaturatingInto<S> for T {
    #[inline]
    fn saturating_into(self) -> S {
        S::saturating_from(self)
    }
}

pub trait WrappingFrom<T> {
    fn wrapping_from(value: T) -> Self;
}

pub trait WrappingInto<T> {
    fn wrapping_into(self) -> T;
}

impl<T, S: WrappingFrom<T>> WrappingInto<S> for T {
    #[inline]
    fn wrapping_into(self) -> S {
        S::wrapping_from(self)
    }
}
