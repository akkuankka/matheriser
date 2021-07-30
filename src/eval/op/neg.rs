use crate::eval::{Number, Radical, Symbolic};
use std::rc::Rc;
use std::ops::Neg;

impl Neg for &Number {
    type Output = Number;
    fn neg(self) -> Self::Output {
        match self.clone() {
            Number::Int(i) => Number::Int(-i),
            Number::Float(f) => Number::Float(-f),
            Number::Symbol(s) => Number::Symbolic(Rc::new(Symbolic {
                coeff: Some(Number::Int(-1)),
                symbol: s,
                constant: None
            })),
            Number::Symbolic(s) => Number::Symbolic(Rc::new(Symbolic {
                coeff: s.coeff.as_ref().map(|x| -x).or(Some(Number::Int(-1))),
                symbol: s.symbol,
                constant: s.constant.as_ref().map(|x| -x)
            })),
            Number::Rational(r) => Number::Rational(-r),
            Number::Radical(r) => Number::Radical(Rc::new(Radical::new( -r.coefficient, r.index, &r.radicand)))
        }
    }
}
