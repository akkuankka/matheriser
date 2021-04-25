use crate::eval::Data;
use std::ops::Sub;

//possibly a little simplistic but alas
impl Sub for Data {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}
