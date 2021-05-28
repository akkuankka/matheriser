/// This function returns `None` if both inputs are `None`, if just one is `Some` it returns that, otherwise, both inputs are some, so it returns `Some(f(a, b))`
pub trait OrMerge<T>
where
    Self: Sized,
{
    type Inner;
    type FOutput;
    type Output;

    fn or_merge(self, f: impl FnOnce(Self::Inner, Self::Inner) -> Self::FOutput, other: T) -> Self::Output;
}

impl<T> OrMerge<Option<T>> for Option<T> {
    type Inner = T;
    type FOutput = T;
    type Output = Self;

    fn or_merge(self, f: impl FnOnce(T, T) -> T, other: Option<T>) -> Self::Output {
        match (self, other) {
            (None, None) => None,
            (None, a) => a,
            (a, None) => a,
            (Some(a), Some(b)) => Some(f(a, b)),
        }
    }
}

impl<T, U> OrMerge<Result<Option<T>, U>> for Option<T> {
   type Inner = T;
    type Output = Result<Option<T>, U>;
    type FOutput = Result<T, U>;
   fn or_merge(self, f: impl FnOnce(T, T) -> Result<T, U>, other: Result<Option<T>, U>) -> Self::Output {
      match other {
          Ok(other) => match (self, other) {
              (None, None) => Ok(None),
              (None, a) => Ok(a),
              (a, None) => Ok(a),
              (Some(a), Some(b)) => match f(a, b) {
                  Ok(c) => Ok(Some(c)),
                  Err(e) => Err(e)
              },
          }
          Err(e) => Err(e)
      }
   } 
}

impl<T> OrMerge<T> for Option<T> {
    type Inner = T;
    type Output = Self;
    type FOutput = T;
    fn or_merge(self, f: impl FnOnce(T, T) -> T, other: T) -> Self {
        match self {
            None => Some(other),
            Some(a) => Some(f(a, other))
        }
    }
}


pub trait Catch
where
    Self: Sized,
{
   type Inner;
   fn catch(self, val: Self::Inner) -> Self; 
}

impl<T> Catch for Option<T> {
    type Inner = T;
    fn catch(self, val: T) -> Self {
        match self {
            Some(val) => None,
            None => None,
            _ => self
        }
    }
}
