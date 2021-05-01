use crate::eval::{Data, DivisibleBy, OrMerge, Radical, Symbolic};
use num::{rational::Ratio, BigInt};
use std::ops::Add;

impl Add for Data {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Self::Int(lhs), Self::Int(rhs)) => Self::Int(lhs + rhs),
            (Self::Int(lhs), Self::Rational(rhs)) => Self::Rational(rhs + lhs),
            (Self::Rational(lhs), Self::Rational(rhs)) => Self::Rational(rhs + lhs),
            (Self::Rational(lhs), Self::Int(rhs)) => Self::Rational(lhs + rhs),
            (Self::Float(lhs), _) => Self::Float(lhs + rhs.as_float().into()),
            (_, Self::Float(rhs)) => Self::Float(self.as_float().into() + rhs),
            (Self::Symbol(sym), _) => Self::Symbolic(Box::new(Symbolic {
                coeff: None,
                symbol: sym,
                constant: Some(rhs),
            })),
            (_, Self::Symbol(sym)) => Self::Symbolic(Box::new(Symbolic {
                coeff: None,
                symbol: sym,
                constant: Some(self),
            })),
            (Self::Symbolic(lcontent), Self::Symbolic(rcontent)) => {
                let Symbolic {
                    coeff: lcoeff,
                    symbol: lsymbol,
                    constant: lconstant,
                } = *lcontent;
                let Symbolic {
                    coeff: rcoeff,
                    symbol: rsymbol,
                    constant: rconstant,
                } = *rcontent;
                if lsymbol == rsymbol {
                    Self::Symbolic(Box::new(Symbolic {
                        coeff: lcoeff.or_merge(|a, b| a + b, rcoeff),
                        symbol: lsymbol,
                        constant: lconstant.or_merge(|a, b| a + b, rconstant),
                    }))
                } else {
                    Self::Symbolic(Box::new(Symbolic {
                        coeff: lcoeff,
                        symbol: lsymbol,
                        constant: lconstant.or_merge(|a, b| a + b, Some(Data::Symbolic(rcontent))),
                    }))
                }
            }
            (Self::Symbolic(content), _) => {
                let Symbolic {
                    coeff,
                    symbol,
                    constant,
                } = *content;
                Self::Symbolic(Box::new(Symbolic {
                    coeff,
                    symbol,
                    constant: constant.or_merge(|a, b| a + b, Some(rhs)),
                }))
            }
            (_, Self::Symbolic(content)) => {
                let Symbolic {
                    coeff,
                    symbol,
                    constant,
                } = *content;
                Self::Symbolic(Box::new(Symbolic {
                    coeff,
                    symbol,
                    constant: constant.or_merge(|a, b| a + b, Some(self)),
                }))
            }
            (Self::Radical(rad), Self::Int(int)) | (Self::Int(int), Self::Radical(rad)) => {
                // assuming the radical is not illformed (i.e. shouldn't exist), this shouldn't yield a pretty radical, therefore it must go to a float ;-;
                Self::Float(rad.as_float() + int as f64)
            }
            (Self::Radical(lhs), Self::Radical(rhs)) => {
                if lhs.index == rhs.index && lhs.radicand == rhs.radicand {
                    Self::Radical(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: lhs.index,
                        radicand: lhs.radicand,
                    })
                }
                // weird edge case: you can manipulate one to look like the other because the indices are divisible.
                else if lhs.index.divisible_by(rhs.index)
                    && lhs.radicand == rhs.radicand.pow(lhs.index / rhs.index)
                {
                    Self::Radical(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: lhs.index,
                        radicand: lhs.radicand,
                    })
                }
                // needs to commute Bruh
                else if rhs.index.divisible_by(lhs.index)
                    && rhs.radicand == lhs.radicand.pow(rhs.index / lhs.index)
                {
                    Self::Radical(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: rhs.index,
                        radicand: rhs.radicand,
                    })
                } else {
                    Self::Float(lhs.as_float() + rhs.as_float())
                }
            }
        }
    }
}
