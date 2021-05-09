use crate::eval::{Data, Radical, Symbolic, SymbolEval, DivisibleBy};
use std::ops::Div;
use num::rational::Ratio;

impl Div for Data {
    type Output = Result<Self, String>;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs == Self::Int(0) {
            Err(String::from("Maths error: Divide by 0"))
        } else if rhs == 1 {
            Ok(self)
        } else {
            match self {
                Self::Int(n) => match rhs {
                    Self::Int(m) => {
                        if n.divisible_by(m) {
                            Ok(Self::Int(n / m))
                        } else {
                            Ok(Self::Rational(Ratio::from((n, m))))
                        }
                    }
                    Self::Float(m) => {
                        Ok(Self::Float(n as f64 / m))
                    }
                    Self::Symbol(m) => {
                        Ok(Self::Float(n as f64 / m.symbol_eval()?))
                    }
                    Self::Symbolic(m) => {
                        Ok(Self::Float(n as f64 / m.as_float()))
                    }
                    Self::Radical(r) => {
                        (self * Self::Radical(r.conjugate())) / *r.radicand 
                    }
                    Self::Rational(m) => {
                        self * Self::Int(*m.denom()) / Self::Int(m.numer())
                    }
                },
                _ => self.as_float() / rhs.as_float()
            }
        }
    }
}
