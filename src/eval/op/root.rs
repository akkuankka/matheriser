use crate::eval::{Data, Radical, Rational, Symbolic};
pub trait NthRoot<RHS = Self>
where
    Self: Sized,
{
    type Output;
    //computes the nth root of a number
    fn nth_root(self, index: RHS) -> Self::Output;
}

impl NthRoot for f64 {
    type Output = Self;
    // yoinked off of rosettacode, does it work ??
    fn nth_root(self, A: f64) -> f64 {
        let p = 1e-9_f64;
        let mut x0 = A / self;
        loop {
            let mut x1 = ((self - 1.0) * x0 + A / f64::powf(x0, self - 1.0)) / self;
            if (x1 - x0).abs() < (x0 * p).abs() {
                return x1;
            };
            x0 = x1
        }
    }
}

impl NthRoot for i64 {
    type Output = Option<Self> {
         
    }
}

impl NthRoot<i64> for Data {
    type Output = Result<Self, String>;
    fn nth_root(self, rhs: i64) -> Output {
        if rhs == 0 {
            Err(String::from(
                "Maths error: cannot take the 0th root of a number",
            ))
        } else {
            let (shouldInvert, index) = (rhs < 0, rhs.abs());
            match self {
                Self::Int(lhs) => {
                    
                }
            }
        }
        Ok(())
    }
}
