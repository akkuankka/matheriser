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
/* big maths time: whether a number is square, cube,
 etc can be determined by whether it can be satisfied
 by a sum of subsequent results of this formula: where P(a:b)
 is the bth element of pascal's triangle row a

 (k + 1)^n - k(n) = P(n:1)*k^(n-1) + P(n:2)*k^(n-2) + ...+ P(n:n-1)*k + 1
 e.g. (k+1)^2 - k^2 = 2k+1 or
      (k+1)^3 - k^3 = 3k^2 + 3k + 1 
 thus if you go through values of k until you either
 exceed or equal your target number you know if it is n-cubic */

/// Generates the non-1 elements of pascals triangle for a row `n` 
fn generate_pascals_row_inners(n: u32) -> Vec<u32> {
    if (n == 0) | (n == 1) {return vec![0]}
    else {
        let use_symmetry_point = (n as f64 / 2.).ceil() as u32;
        let mut result = vec![n];
        let mut reflection_counter = 1_u32;
        for i in 2..=n {
           let prev = *result.last().unwrap(); // we put something in the vec just then
           let current = prev * ((n + 1 - i)/i);
           result.push(current);
        }
        result
    }
}

fn is_n_dim_cube(n: u32, dimension: u32) -> bool {
    
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
