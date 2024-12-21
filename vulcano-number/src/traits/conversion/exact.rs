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
