use crate::eval::{op::pow::Pow, Data, DivisibleBy, OrMerge, Radical, Symbolic};
use std::ops::Add;

impl Add for Data {
    type Output = Result<Self, String>;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Int(lhs), Self::Int(rhs)) => Ok(Self::Int(lhs + rhs)),
            (Self::Int(lhs), Self::Rational(rhs)) => Ok(Self::Rational(rhs + lhs)),
            (Self::Rational(lhs), Self::Rational(rhs)) => Ok(Self::Rational(rhs + lhs)),
            (Self::Rational(lhs), Self::Int(rhs)) => Ok(Self::Rational(lhs + rhs)),
            (Self::Float(lhs), _) => Ok(Self::Float(lhs + f64::from(rhs))),
            (_, Self::Float(rhs)) => Ok(Self::Float(f64::from(self) + rhs)),
            (Self::Symbol(sym), _) => Ok(Self::Symbolic(Box::new(Symbolic {
                coeff: None,
                symbol: sym,
                constant: Some(rhs),
            }))),
            (_, Self::Symbol(sym)) => Ok(Self::Symbolic(Box::new(Symbolic {
                coeff: None,
                symbol: sym,
                constant: Some(self),
            }))),
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
                    Ok(Self::Symbolic(Box::new(Symbolic {
                        coeff: lcoeff.or_merge(|a, b| a + b, Ok(rcoeff))?,
                        symbol: lsymbol,
                        constant: lconstant.or_merge(|a, b| a + b, Ok(rconstant))?,
                    })))
                } else {
                    Ok(Self::Symbolic(Box::new(Symbolic {
                        coeff: lcoeff,
                        symbol: lsymbol,
                        constant: lconstant
                            .or_merge(|a, b| a + b, Ok(Some(Data::Symbolic(rcontent))))?,
                    })))
                }
            }
            (Self::Symbolic(content), _) => {
                let Symbolic {
                    coeff,
                    symbol,
                    constant,
                } = *content;
                Ok(Self::Symbolic(Box::new(Symbolic {
                    coeff,
                    symbol,
                    constant: constant.or_merge(|a, b| a + b, Ok(Some(rhs)))?,
                })))
            }
            (_, Self::Symbolic(content)) => {
                let Symbolic {
                    coeff,
                    symbol,
                    constant,
                } = *content;
                Ok(Self::Symbolic(Box::new(Symbolic {
                    coeff,
                    symbol,
                    constant: constant.or_merge(|a, b| a + b, Ok(Some(self)))?,
                })))
            }
            (Self::Radical(rad), Self::Int(int)) | (Self::Int(int), Self::Radical(rad)) => {
                // assuming the radical is not illformed (i.e. shouldn't exist), this shouldn't yield a pretty radical, therefore it must go to a float ;-;
                Ok(Self::Float(rad.as_float() + int as f64))
            }
            (Self::Radical(lhs), Self::Radical(rhs)) => {
                if lhs.index == rhs.index && lhs.radicand == rhs.radicand {
                    Ok(Self::Radical(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: lhs.index,
                        radicand: lhs.radicand,
                    }))
                }
                // weird edge case: you can manipulate one to look like the other because the indices are divisible.
                else if lhs.index.divisible_by(rhs.index)
                    && lhs.radicand
                        == rhs
                            .radicand
                            .pow(((lhs.index / rhs.index) as i64).into())?
                            .into()
                {
                    Ok(Self::Radical(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: lhs.index,
                        radicand: lhs.radicand,
                    }))
                }
                // needs to commute Bruh
                else if rhs.index.divisible_by(lhs.index)
                    && rhs.radicand
                        == lhs
                            .radicand
                            .pow(((rhs.index / lhs.index) as i64).into())?
                            .into()
                {
                    Ok(Self::Radical(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: rhs.index,
                        radicand: rhs.radicand,
                    }))
                } else {
                    Ok(Self::Float(lhs.as_float() + rhs.as_float()))
                }
            }
        }
    }
}
