use crate::eval::{Data, DivisibleBy, Radical, Symbolic, OrMerge};
use num::rational::Ratio;
use std::ops::Neg;

impl Neg for Data {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Int(i) => Self::Int(-i),
            Self::Float(f) => Self::Float(-f),
            Self::Symbol(s) => Self::Symbolic(Box::new(Symbolic {
                coeff: Some(Self::Int(-1)),
                symbol: s,
                constant: None
            })),
            Self::Symbolic(s) => Self::Symbolic(Box::new(Symbolic {
                coeff: s.coeff.map(|x| -x).or(Some(Data::Int(-1))),
                symbol: s.symbol,
                constant: s.coeff.map(|x| -x)
            })),
            Self::Rational(r) => Self::Rational(-r),
            Self::Radical(r) => Self::Radical(Radical::new( -r.coefficient, r.index, r.radicand))
        }
    }
}
