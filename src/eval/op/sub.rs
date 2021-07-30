use crate::eval::Number;
use super::Add;

trait Sub {
    type Output;
    fn sub(&self, rhs: &Self) -> Self::Output;
}

//possibly a little simplistic but alas
impl Sub for Number {
    type Output = Result<Self, String>;
    fn sub(&self, rhs: &Self) -> Self::Output {
        self.add(&-rhs)
    }
}
