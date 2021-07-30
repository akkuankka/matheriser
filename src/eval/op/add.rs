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
        match (self, rhs) { 
            (Number::Int(lhs), Number::Int(rhs)) => Ok(Number::Int(lhs + rhs)),
            (Number::Int(lhs), Number::Rational(rhs)) => Ok(Number::Rational(rhs + lhs)),
            (Number::Rational(lhs), Number::Rational(rhs)) => Ok(Number::Rational(rhs + lhs)),
            (Number::Rational(lhs), Number::Int(rhs)) => Ok(Number::Rational(lhs + rhs)),
            (Number::Float(lhs), a) => Ok(Number::Float(lhs + f64::try_from(a.clone())?)),
            (a, Number::Float(rhs)) => Ok(Number::Float(f64::try_from(a.clone())? + rhs)),
            (Number::Symbol(sym), a) => Ok(Number::Symbolic(Rc::new(Symbolic {
                coeff: None,
                symbol: *sym,
                constant: Some(a.clone()),
            }))),
            (a, Number::Symbol(sym)) => Ok(Number::Symbolic(Rc::new(Symbolic {
                coeff: None,
                symbol: *sym,
                constant: Some(a.clone()),
            }))),
            (Number::Symbolic(lcontent), Number::Symbolic(rcontent)) => {
                let Symbolic {
                    coeff: ref lcoeff,
                    symbol: lsymbol,
                    constant: ref lconstant,
                } = **lcontent;
                let Symbolic {
                    coeff: ref rcoeff,
                    symbol: rsymbol,
                    constant: ref rconstant,
                } = **lcontent;
                if lsymbol == rsymbol {
                    Ok(Number::Symbolic(Rc::new(Symbolic {
                        coeff: lcoeff.clone().or_merge(|a, b| a.add(&b), Ok(rcoeff.clone()))?,
                        symbol: lsymbol,
                        constant: lconstant.clone().or_merge(|a, b| a.add(&b), Ok(rconstant.clone()))?,
                    })))
                } else {
                    Ok(Number::Symbolic(Rc::new(Symbolic {
                        coeff: lcoeff.clone(),
                        symbol: lsymbol,
                        constant: lconstant.clone()
                            .or_merge(|a, b| a.add(&b), Ok(Some(Number::Symbolic(rcontent.clone()))))?,
                    })))
                }
            }
            (Number::Symbolic(content), a) => {
                let Symbolic {
                    ref coeff,
                    symbol,
                    ref constant,
                } = **content;
                Ok(Number::Symbolic(Rc::new(Symbolic {
                    coeff: coeff.clone(),
                    symbol,
                    constant: constant.clone().or_merge(|a, b| a.add(&b), Ok(Some(a.clone())))?,
                })))
            }
            (a, Number::Symbolic(content)) => {
                let Symbolic {
                    ref coeff,
                    symbol,
                    ref constant,
                } = **content;
                Ok(Number::Symbolic(Rc::new(Symbolic {
                    coeff: coeff.clone(),
                    symbol,
                    constant: constant.clone().or_merge(|a, b| a.add(&b), Ok(Some(a.clone())))?,
                })))
            }
            (Number::Radical(rad), Number::Int(int)) | (Number::Int(int), Number::Radical(rad)) => {
                // assuming the radical is not illformed (i.e. shouldn't exist), this shouldn't yield a pretty radical, therefore it must go to a float ;-;
                Ok(Number::Float(rad.as_float()? + *int as f64))
            }
            (Number::Radical(lhs), Number::Radical(rhs)) => {
                if lhs.index == rhs.index && lhs.radicand == rhs.radicand {
                    Ok(Number::Radical(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: lhs.index,
                        radicand: lhs.radicand.clone(),
                    }.into()))
                }
                // weird edge case: you can manipulate one to look like the other because the indices are divisible.
                else if lhs.index.divisible_by(rhs.index)
                    && lhs.radicand
                        == rhs
                            .radicand
                            .pow(&Number::from((lhs.index / rhs.index) as i64))?
                            .into()
                {
                    Ok(Number::Radical(Rc::new(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: lhs.index,
                        radicand: lhs.radicand.clone(),
                    })))
                }
                // needs to commute Bruh
                else if rhs.index.divisible_by(lhs.index)
                    && rhs.radicand
                        == lhs
                            .radicand
                            .pow(&Number::from((rhs.index / lhs.index) as i64))?
                {
                    Ok(Number::Radical(Rc::new(Radical {
                        coefficient: lhs.coefficient + rhs.coefficient,
                        index: rhs.index,
                        radicand: rhs.radicand.clone(),
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
