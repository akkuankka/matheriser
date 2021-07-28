use crate::eval::{op::pow::Pow, DivisibleBy, Number, Radical, Symbolic};
use crate::util::option::{Catch, OrMerge};
use std::convert::TryFrom;
use std::rc::Rc;

use super::Add;

pub trait Mul {
    type Output;
    fn mul(&self, rhs: &Self) -> Self::Output;
}

impl Mul for Number {
    type Output = Result<Number, String>;
    fn mul(&self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            // Easy ones first: same type so we get commutation free
            (Number::Int(a), Number::Int(b)) => Ok(Number::Int(a * b)),
            (Number::Float(a), Number::Float(b)) => Ok(Number::Float(a * b)),
            // We can't rely entirely on the implementation that num provides, because when the denominator = 1 it needs to be an Int
            (Number::Rational(a), Number::Rational(b)) => {
                let result = a * b;
                if *result.denom() == 1 {
                    Ok(Number::Int(*result.numer()))
                } else {
                    Ok(Number::Rational(result))
                }
            }
            // Ok now we have to implement this ourself
            (Number::Radical(a), Number::Radical(b)) => {
                Ok(if a.index == b.index {
                    Number::Radical(Rc::new(Radical::new(
                        a.coefficient * b.coefficient,
                        a.index,
                        &a.radicand.mul(&b.radicand)?,
                    )))
                } else if a.index.divisible_by(b.index) {
                    Number::Radical(Rc::new(Radical::new(
                        a.coefficient * b.coefficient,
                        a.index,
                        &a.radicand.mul(
                            &b.radicand
                                .pow(&Number::from(a.index as i64 / b.index as i64))?,
                        )?,
                    )))
                } else if b.index.divisible_by(a.index) {
                    Number::Radical(Rc::new(Radical::new(
                        b.coefficient * a.coefficient,
                        b.index,
                        &b.radicand.mul(
                            &a.radicand
                                .pow(&Number::from(b.index as i64 / a.index as i64))?,
                        )?,
                    )))
                } else if a.radicand == b.radicand {
                    Number::Radical(Rc::new(Radical::new(
                        b.coefficient * a.coefficient,
                        a.index + b.index,
                        &a.radicand,
                    )))
                }
                // I can't think of any further improvements so I guess we just go
                else {
                    Number::Float(a.as_float()? * b.as_float()?)
                })
            }
            (Number::Symbol(a), Number::Symbol(b)) => Ok(Number::Symbolic(
                Symbolic {
                    coeff: Some(Number::Symbol(*a)),
                    symbol: *b,
                    constant: None,
                }
                .into(),
            )),
            (Number::Symbolic(a), Number::Symbolic(b)) => {
                match (b.coeff.clone(), b.constant.clone()) {
                    (None, None) => {
                        println!("This is embarassing, I messed up some algebra, it's fine though");
                        Ok(Number::Symbolic(
                            Symbolic {
                                coeff: Some(Number::Symbol(a.symbol)),
                                symbol: b.symbol,
                                constant: None,
                            }
                            .into(),
                        ))
                    }
                    (Some(k), None) => Ok(Number::Symbolic(
                        Symbolic {
                            coeff: a.coeff.or_merge(|x, y| x.mul(&y), Ok(Some(k)))?.or_merge(
                                |x, y| x.mul(&y),
                                Ok(Some(Number::Symbol(b.symbol))),
                            )?,
                            symbol: a.symbol,
                            constant: match a.constant {
                                None => None,
                                Some(x) => Some(
                                    (x.mul(&k)).and_then(|y| y.mul(&b.symbol.into()))?,
                                ),
                            },
                        }
                        .into(),
                    )),
                    (b_coeff, b_constant) => {
                        // needs testing, IDK if this actually does what I think it does
                        Number::Symbolic(
                            Symbolic {
                                coeff: a
                                    .coeff
                                    .clone()
                                    .or_merge(|x, y| x.mul(&y), Ok(b_coeff))?
                                    .or_merge(|x, y| x.mul(&y), Ok(Some(b.symbol.into())))?
                                    .catch(Number::Int(1)),
                                symbol: a.symbol.clone(),
                                constant:
                                match (
                                    match a.constant {
                                        // extremely weird syntactic hackery due to not being able to use map
                                        None => None,
                                        Some(x) => Some(x.mul(&b.symbol.into())?), // if it is Some(x), multiply it by symbol
                                    },
                                    b_coeff,
                                ) {
                                    (None, _) => None,
                                    (Some(t), None) => Some(t),
                                    (Some(l), Some(r)) => Some(l.mul(&r)?), // if it is Some(x), multiply it by coefficient
                                }
                                .catch(Number::Int(0)),
                            }
                            .into(),
                        ).add(&Number::Symbolic(
                            Symbolic {
                                coeff: a
                                    .coeff
                                    .or_merge(|x, y| x.mul(&y), Ok(b_constant.clone()))?
                                    .catch(Number::Int(1)),
                                symbol: a.symbol,
                                constant: match (a.constant, b_constant) {
                                    (None, _) => None,
                                    (Some(t), None) => Some(t),
                                    (Some(l), Some(r)) => Some(l.mul(&r)?),
                                }
                                .catch(Number::Int(0)),
                            }
                            .into(),
                        )
                        )
                    }
                }
            } // now that all the single-type operations are done, the two sided ones
            (Number::Float(flt), a) => Ok(Number::Float(flt * f64::try_from(*a)?)), // get floats out of the way because they're bad
            (a, Number::Float(flt)) => Ok(Number::Float(flt * f64::try_from(*a)?)),
            (Number::Symbolic(syc), Number::Int(int))
            | (Number::Int(int), Number::Symbolic(syc)) => {
                Ok(Number::Symbolic(
                    //next symbolics because they're specific
                    Symbolic {
                        coeff: syc
                            .coeff
                            .or_merge(|x, y| x.mul(&y), Ok(Some(Number::from(*int))))?
                            .catch(Number::Int(1)),
                        symbol: syc.symbol,
                        constant: match syc.constant.map(|x| x.mul(&Number::from(*int))) {
                            None => None,
                            Some(t) => Some(t?),
                        }
                        .catch(Number::Int(0)),
                    }
                    .into(),
                ))
            }
            (Number::Symbolic(syc), Number::Symbol(sym))
            | (Number::Symbol(sym), Number::Symbolic(syc)) => Ok(Number::Symbolic(
                Symbolic {
                    coeff: syc
                        .coeff
                        .or_merge(|x, y| x.mul(&y), Ok(Some(Number::Symbol(*sym))))?,
                    symbol: syc.symbol,
                    constant: match syc.constant.map(|x| x.mul(&Number::Symbol(*sym))) {
                        None => None,
                        Some(t) => Some(t?),
                    },
                }
                .into(),
            )),
            (Number::Symbolic(syc), Number::Rational(rat))
            | (Number::Rational(rat), Number::Symbolic(syc)) => Ok(Number::Symbolic(
                Symbolic {
                    coeff: syc
                        .coeff
                        .or_merge(|x, y| x.mul(&y), Ok(Some(Number::Rational(*rat))))?
                        .catch(Number::Int(1)),
                    symbol: syc.symbol,
                    constant: match syc.constant.map(|x| x.mul(&Number::Rational(*rat))) {
                        None => None,
                        Some(t) => Some(t?),
                    }
                    .catch(Number::Int(0)),
                }
                .into(),
            )),
            (Number::Symbolic(syc), Number::Radical(rad))
            | (Number::Radical(rad), Number::Symbolic(syc)) => Ok(Number::Symbolic(
                Symbolic {
                    coeff: syc
                        .coeff
                        .or_merge(
                            |x, y| x.mul(&y),
                            Ok(Some(Number::Radical(rad.clone()))),
                        )?
                        .catch(Number::Int(1)),
                    symbol: syc.symbol,
                    constant: match syc.constant.map(|x| x.mul(&Number::Radical(*rad))) {
                        None => None,
                        Some(t) => Some(t?),
                    }
                    .catch(Number::Int(0)),
                }
                .into(),
            )),
            (Number::Int(int), Number::Symbol(sym)) | (Number::Symbol(sym), Number::Int(int)) => {
                Ok(Number::Symbolic(
                    Symbolic {
                        coeff: Some(Number::Int(*int)),
                        symbol: *sym,
                        constant: None,
                    }
                    .into(),
                ))
            }
            (Number::Int(int), Number::Rational(rat))
            | (Number::Rational(rat), Number::Int(int)) => {
                let result = rat * int;
                if *result.denom() == 1 {
                    Ok(Number::Int(*result.numer()))
                } else {
                    Ok(Number::Rational(result))
                }
            }
            (Number::Int(int), Number::Radical(rad)) | (Number::Radical(rad), Number::Int(int)) => {
                Ok(Number::Radical(Rc::new(Radical::new(
                    rad.coefficient * int,
                    rad.index,
                    &rad.radicand,
                ))))
            }
            (Number::Rational(rat), Number::Radical(rad))
            | (Number::Radical(rad), Number::Rational(rat)) => Ok(Number::Radical(Rc::new(Radical::new(
                rad.coefficient * rat,
                rad.index,
                &rad.radicand,
            )))),
            (a, Number::Symbol(s)) | (Number::Symbol(s), a) => Ok(Number::Symbolic(
                Symbolic {
                    coeff: Some(*a),
                    symbol: *s,
                    constant: None,
                }
                .into(),
            )),
        }
    }
}
