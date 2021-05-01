/// This function returns `None` if both inputs are `None`, if just one is `Some` it returns that, otherwise, both inputs are some, so it returns `Some(f(a, b))`
pub trait OrMerge<T>
where
    Self: Sized,
{
    type Inner;

    fn or_merge(self, f: impl FnOnce(Self::Inner, Self::Inner) -> Self::Inner, other: T) -> Self;
}

impl<T> OrMerge<Option<T>> for Option<T> {
    type Inner = T;
    fn or_merge(self, f: impl FnOnce(T, T) -> T, other: Option<T>) -> Self {
        match (self, other) {
            (None, None) => None,
            (None, a) => a,
            (a, None) => a,
            (Some(a), Some(b)) => Some(f(a, b)),
        }
    }
}

impl<T> OrMerge<T> for Option<T> {
    type Inner = T;
    fn or_merge(self, f: impl FnOnce(T, T) -> T, other: T) -> Self {
        match self {
            None => Some(other),
            Some(a) => Some(f(a, other))
        }
    }
}
