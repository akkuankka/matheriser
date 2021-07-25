use crate::eval::Number;
use std::ops::Sub;

//possibly a little simplistic but alas
impl Sub for Number {
    type Output = Result<Self, String>;
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}
