use crate::eval::{Number, DivisibleBy, Radical};
use num::rational::Ratio;
use std::convert::TryInto;
use std::rc::Rc;

use super::Mul;
pub trait NthRoot<RHS = Self>
where
    Self: Sized,
    RHS: Copy // this is copy because it's passed by value
{
    type Output;
    //computes the nth root of a number
    fn nth_root(&self, index: RHS) -> Self::Output;
}

impl NthRoot<i64> for f64 {
    type Output = Option<Self>;
    fn nth_root(&self, index: i64) -> Self::Output {
        if index == 0 {
            None
        } else if index % 2 == 0 && *self < 0. {
            None
        } else {
            let (should_negate, abs_self) = (*self < 0., self.abs());
            Some({ |x: f64| if should_negate { -x } else { x } }(
                abs_self.powf(1. / index as f64),
            ))
        }
    }
}

impl NthRoot<f64> for f64 {
    type Output = Option<Self>;
    fn nth_root(&self, index: f64) -> Self::Output {
        if index == 0. || *self < 0. {
            None
        } else {
            Some(self.powf(1. / index))
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
exceed or equal your target number you know if the target is n-cubic */

/// Generates the non-1 elements of pascals triangle for a row `n`
fn generate_pascals_row_inners(n: u32) -> Vec<u32> {
    if (n == 0) | (n == 1) {
        return vec![0];
    } else {
        let mut result = vec![n];
        for i in 2..=n {
            let prev = *result.last().unwrap(); // we put something in the vec just then
            let plusses = n - i;
            let numerator = (plusses + 1) * prev;
            let current = numerator / i;
            if current == 1 {
                continue;
            } else {
                result.push(current)
            }
        }
        result
    }
}

// I thought that this would be useful to find what numbers would be
// good candidates for brute-force squaring, but as it turns out, this
// is a better algorithm for finding the root itself !
// fn is_n_dim_cube(n: u32, dimension: u32) -> bool {
//     // this is a very un-functional way of implementing this
//     let increment_for_dimension_formula = |k: u32| {
//         // but I could not care less
//         let coeffs = generate_pascals_row_inners(dimension);
//         let mut accumulator = 0;
//         for (i, coeff) in coeffs.into_iter().enumerate() {
//             accumulator += coeff * k.pow(dimension - 1 - i as u32)
//         }
//         accumulator + 1
//     };
//     let mut so_far = 0_u32;
//     for i in 0..n {
//         so_far += increment_for_dimension_formula(i);
//         if so_far == n {return true;}
//         else if so_far > n {return false;}
//     };
//     false // this should actually never happen
// }

impl NthRoot for u32 {
    type Output = Option<Self>;
    fn nth_root(&self, dimension: u32) -> Self::Output {
        // this is a very un-functional way of implementing this
        let increment_for_dimension_formula = |k: u32, coeffs: &Vec<u32>| {
            // but I could not care less
            let mut accumulator = 0;
            for (i, coeff) in coeffs.into_iter().enumerate() {
                accumulator += coeff * k.pow(dimension - 1 - i as u32)
            }
            accumulator + 1
        };
        let mut so_far = 0_u32;
        let inners = generate_pascals_row_inners(dimension);
        for i in 0..*self {
            let inc = increment_for_dimension_formula(i, &inners);
            so_far += inc;
            if so_far == *self {
                return Some(i + 1);
            } else if so_far > *self {
                return None;
            }
        }
        None // this should actually never happen
    }
}

impl NthRoot<i64> for Number {
    type Output = Result<Number, String>;
    fn nth_root(&self, rhs: i64) -> Self::Output {
        if rhs == 0 {
            Err(String::from(
                "Maths error: cannot take the 0th root of a number",
            ))
        } else {
            Ok({
                let (should_invert, index) = (rhs < 0, rhs.abs() as u32);
                let mut should_negate = false;
                match self {
                    Self::Int(lhs) => {
                        // If it is an int then 1:
                        if *lhs < 0 {
                            // we need to check that we're not taking the square/4th etc root of a negative number
                            if rhs.divisible_by(2) {
                                return Err("Non-real error: even root of a negative number".into());
                            } else {
                                should_negate = true; // in that case we just negate the output of it as if it were a positive number
                            }
                        }
                        let abs_lhs = lhs.abs();
                        match (abs_lhs as u32).nth_root(index) {
                            // now that that's sorted out, does the root resolve to an integer?
                            Some(n) => Self::Int(n as i64 * if should_negate { -1 } else { 1 }), // if so, return that (negated as necessary)
                            None => Self::Radical(Rc::new(Radical::new_raw(
                                if should_invert {
                                    Ratio::from(-1)
                                } else {
                                    Ratio::from(1)
                                },
                                index,
                                self,
                            ))),
                        }
                    }
                    Self::Float(n) => {
                        if *n < 0. {
                            // we need to check that we're not taking the square/4th etc root of a negative number
                            if rhs.divisible_by(2) {
                                return Err("Non-real error: even root of a negative number".into());
                            } else {
                                should_negate = true; // in that case we just negate the output of it as if it were a positive number
                            }
                        }
                        Self::Radical(Rc::new(Radical::new_raw(
                            if should_negate {
                                Ratio::from(-1)
                            } else {
                                Ratio::from(1)
                            },
                            index,
                            self,
                        )))
                    }
                    Self::Rational(rat) => {
                        if rat < &Ratio::from(0) {
                            // we need to check that we're not taking the square/4th etc root of a negative number
                            if rhs.divisible_by(2) {
                                return Err(
                                    "Non-real error: even root of a negative number".to_string()
                                );
                            } else {
                                should_negate = true; // in that case we just negate the output of it as if it were a positive number
                            }
                        }
                        Self::Radical(Rc::new(Radical::new_raw(
                            Ratio::from((if should_negate { 1 } else { -1 }, *rat.denom())),
                            index,
                            &Self::Int(*rat.denom() * *rat.numer()),
                        )))
                    }
                    Self::Radical(rad) => {
                        let result =
                            Radical::new_raw(rad.coefficient, rad.index * index, &rad.radicand);
                        if result.index.divisible_by(2) {
                            if result.radicand < Self::Int(0) {
                                return Err(
                                    "Non-real error: even root of a negative number".to_string()
                                );
                            } else {
                                should_negate = true;
                            }
                        }
                        if should_negate {
                            Self::Radical(Rc::new(result)).mul(&Self::Int(-1))?
                        } else {
                            Self::Radical(Rc::new(result))
                        }
                    }
                    Self::Symbol(s) => {
                        let f: f64 = Self::Symbol(*s).try_into()?;
                        if f < 0. {
                            // we need to check that we're not taking the square/4th etc root of a negative number
                            if rhs.divisible_by(2) {
                                return Err(
                                    "Non-real error: even root of a negative number".to_string()
                                );
                            } else {
                                should_negate = true; // in that case we just negate the output of it as if it were a positive number
                            }
                        }
                        Self::Radical(Rc::new(Radical::new_raw(
                            Ratio::from(if should_negate { 1 } else { -1 }),
                            index,
                            &Self::Symbol(*s),
                        )))
                    }
                    Self::Symbolic(s) => {
                        let f: f64 = Self::Symbolic(s.clone()).try_into()?;
                        if f < 0. {
                            // we need to check that we're not taking the square/4th etc root of a negative number
                            if rhs.divisible_by(2) {
                                return Err(
                                    "Non-real error: even root of a negative number".to_string()
                                );
                            } else {
                                should_negate = true; // in that case we just negate the output of it as if it were a positive number
                            }
                        }
                        Self::Radical(Rc::new(Radical::new_raw(
                            Ratio::from(if should_negate { 1 } else { -1 }),
                            index,
                            &Self::Symbolic(s.clone()),
                        )))
                    }
                }
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pascals_triangle() {
        assert_eq!(generate_pascals_row_inners(2), vec![2]);
        assert_eq!(generate_pascals_row_inners(4), vec![4, 6, 4])
    }

    #[test]
    fn root_u32() {
        assert_eq!(125_u32.nth_root(3), Some(5_u32));
        assert_eq!(9.nth_root(2), Some(3_u32))
    }
}
