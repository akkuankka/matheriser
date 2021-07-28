use crate::eval::{op::pow::Pow, ratio_as_float, Number, DivisibleBy, OrMerge, Radical, Symbolic};
use std::convert::TryFrom;
use std::rc::Rc;

pub trait Add<RHS = Self> {
    type Output;
    fn add(&self, rhs: &RHS) -> Self::Output;
}

impl Add for Number {
    type Output = Result<Self, String>;
    fn add(&self, rhs: &Self) -> Self::Output {
        match (self.clone(), rhs.clone()) { // this is not cheeky -- it's either copy or an Rc
            (Number::Int(lhs), Number::Int(rhs)) => Ok(Number::Int(lhs + rhs)),
            (Number::Int(lhs), Number::Rational(rhs)) => Ok(Number::Rational(rhs + lhs)),
            (Number::Rational(lhs), Number::Rational(rhs)) => Ok(Number::Rational(rhs + lhs)),
            (Number::Rational(lhs), Number::Int(rhs)) => Ok(Number::Rational(lhs + rhs)),
            (Number::Float(lhs), a) => Ok(Number::Float(lhs + f64::try_from(a)?)),
            (a, Number::Float(rhs)) => Ok(Number::Float(f64::try_from(a)? + rhs)),
            (Number::Symbol(sym), a) => Ok(Number::Symbolic(Rc::new(Symbolic {
                coeff: None,
                symbol: sym,
                constant: Some(a),
            }))),
            (a, Number::Symbol(sym)) => Ok(Number::Symbolic(Rc::new(Symbolic {
                coeff: None,
                symbol: sym,
                constant: Some(a),
            }))),
            (Number::Symbolic(lcontent), Number::Symbolic(rcontent)) => {
                let Symbolic {
                    coeff: lcoeff,
                    symbol: lsymbol,
                    constant: lconstant,
                } = *lcontent;
                let Symbolic {
                    coeff: rcoeff,
                    symbol: rsymbol,
                    constant: rconstant,
                } = *rcontent.clone();
                if lsymbol == rsymbol {
                    Ok(Number::Symbolic(Rc::new(Symbolic {
                        coeff: lcoeff.or_merge(|a, b| a.add(&b), Ok(rcoeff))?,
                        symbol: lsymbol,
                        constant: lconstant.or_merge(|a, b| a.add(&b), Ok(rconstant))?,
                    })))
                } else {
                    Ok(Number::Symbolic(Rc::new(Symbolic {
                        coeff: lcoeff,
                        symbol: lsymbol,
                        constant: lconstant
                            .or_merge(|a, b| a.add(&b), Ok(Some(Number::Symbolic(rcontent))))?,
                    })))
                }
            }
            (Number::Symbolic(content), a) => {
                let Symbolic {
                    coeff,
                    symbol,
                    constant,
                } = *content;
                Ok(Number::Symbolic(Rc::new(Symbolic {
                    coeff,
                    symbol,
                    constant: constant.or_merge(|a, b| a.add(&b), Ok(Some(a)))?,
                })))
            }
            (a, Number::Symbolic(content)) => {
                let Symbolic {
                    coeff,
                    symbol,
                    constant,
                } = *content;
                Ok(Number::Symbolic(Rc::new(Symbolic {
                    coeff,
                    symbol,
                    constant: constant.or_merge(|a, b| a.add(&b), Ok(Some(a)))?,
                })))
            }
            (Number::Radical(rad), Number::Int(int)) | (Number::Int(int), Number::Radical(rad)) => {
                // assuming the radical is not illformed (i.e. shouldn't exist), this shouldn't yield a pretty radical, therefore it must go to a float ;-;
                Ok(Number::Float(rad.as_float()? + int as f64))
            }
            (Number::Radical(lhs), Number::Radical(rhs)) => {
                if lhs.index == rhs.index && lhs.radicand == rhs.radicand {
                    Ok(Number::Radical(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: lhs.index,
                        radicand: lhs.radicand,
                    }.into()))
                }
                // weird edge case: you can manipulate one to look like the other because the indices are divisible.
                else if lhs.index.divisible_by(rhs.index)
                    && lhs.radicand
                        == rhs
                            .radicand
                            .clone()
                            .pow(((lhs.index / rhs.index) as i64).into())?
                            .into()
                {
                    Ok(Number::Radical(Rc::new(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: lhs.index,
                        radicand: lhs.radicand,
                    })))
                }
                // needs to commute Bruh
                else if rhs.index.divisible_by(lhs.index)
                    && rhs.radicand
                        == &lhs
                            .radicand
                            .pow(((rhs.index / lhs.index) as i64).into())?
                            .into()
                {
                    Ok(Number::Radical(Rc::new(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: rhs.index,
                        radicand: rhs.radicand,
                    })))
                } else {
                    Ok(Number::Float(lhs.as_float()? + rhs.as_float()?))
                }
            }
            (Number::Radical(rad), Number::Rational(rat))
            | (Number::Rational(rat), Number::Radical(rad)) => {
                Ok(Number::Float(rad.as_float()? + ratio_as_float(*rat)))
            }
        }
    }
}
