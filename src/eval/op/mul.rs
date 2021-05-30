use crate::eval::{op::pow::Pow, Data, DivisibleBy, Radical, Symbolic};
use crate::util::option::{Catch, OrMerge};
use std::convert::TryFrom;
use std::ops::Mul;

impl Mul for Data {
    type Output = Result<Data, String>;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Easy ones first: same type so we get commutation free
            (Self::Int(a), Self::Int(b)) => Ok(Self::Int(a * b)),
            (Self::Float(a), Self::Float(b)) => Ok(Self::Float(a * b)),
            // We can't rely entirely on the implementation that num provides, because when the denominator = 1 it needs to be an Int
            (Self::Rational(a), Self::Rational(b)) => {
                let result = a * b;
                if *result.denom() == 1 {
                    Ok(Self::Int(*result.numer()))
                } else {
                    Ok(Self::Rational(result))
                }
            }
            // Ok now we have to implement this ourself
            (Self::Radical(a), Self::Radical(b)) => {
                Ok(if a.index == b.index {
                    Self::Radical(Radical::new(
                        a.coefficient * b.coefficient,
                        a.index,
                        Box::new((*a.radicand * *b.radicand)?),
                    ))
                } else if a.index.divisible_by(b.index) {
                    Self::Radical(Radical::new(
                        a.coefficient * b.coefficient,
                        a.index,
                        Box::new(
                            (*a.radicand
                                * b.radicand
                                    .pow(Data::from(a.index as i64 / b.index as i64))?)?,
                        ),
                    ))
                } else if b.index.divisible_by(a.index) {
                    Self::Radical(Radical::new(
                        b.coefficient * a.coefficient,
                        b.index,
                        Box::new(
                            (*b.radicand
                                * a.radicand
                                    .pow(Data::from(b.index as i64 / a.index as i64))?)?,
                        ),
                    ))
                } else if *a.radicand == *b.radicand {
                    Self::Radical(Radical::new(
                        b.coefficient * a.coefficient,
                        a.index + b.index,
                        a.radicand,
                    ))
                }
                // I can't think of any further improvements so I guess we just go
                else {
                    Self::Float(a.as_float()? * b.as_float()?)
                })
            }
            (Self::Symbol(a), Self::Symbol(b)) => Ok(Self::Symbolic(
                Symbolic {
                    coeff: Some(Self::Symbol(a)),
                    symbol: b,
                    constant: None,
                }
                .into(),
            )),
            (Self::Symbolic(a), Self::Symbolic(b)) => match (b.coeff.clone(), b.constant.clone()) {
                (None, None) => {
                    println!("This is embarassing, I messed up some algebra, it's fine though");
                    Ok(Self::Symbolic(
                        Symbolic {
                            coeff: Some(Self::Symbol(a.symbol)),
                            symbol: b.symbol,
                            constant: None,
                        }
                        .into(),
                    ))
                }
                (Some(k), None) => Ok(Self::Symbolic(
                    Symbolic {
                        coeff: a
                            .coeff
                            .or_merge(|x, y| x * y, Ok(Some(k.clone())))?
                            .or_merge(|x, y| x * y, Ok(Some(Self::Symbol(b.symbol.clone()))))?,
                        symbol: a.symbol,
                        constant: match a.constant {
                            None => None,
                            Some(x) => Some((x * k).and_then(|y| y * b.symbol.into())?),
                        },
                    }
                    .into(),
                )),
                (b_coeff, b_constant) => {
                    // needs testing, IDK if this actually does what I think it does
                    Self::Symbolic(
                        Symbolic {
                            coeff: a
                                .coeff
                                .clone()
                                .or_merge(|x, y| x * y, Ok(b_coeff.clone()))?
                                .or_merge(|x, y| x * y, Ok(Some(b.symbol.clone().into())))?
                                .catch(Self::Int(1)),
                            symbol: a.symbol.clone(),
                            constant: match (
                                match a.constant.clone() {
                                    // extremely weird syntactic hackery due to not being able to use map
                                    None => None,
                                    Some(x) => Some((x * b.symbol.into())?), // if it is Some(x), multiply it by symbol
                                },
                                b_coeff,
                            ) {
                                (None, _) => None,
                                (Some(t), None) => Some(t),
                                (Some(l), Some(r)) => Some((l * r)?), // if it is Some(x), multiply it by coefficient
                            }
                            .catch(Self::Int(0)),
                        }
                        .into(),
                    ) + Self::Symbolic(
                        Symbolic {
                            coeff: a
                                .coeff
                                .or_merge(|x, y| x * y, Ok(b_constant.clone()))?
                                .catch(Self::Int(1)),
                            symbol: a.symbol,
                            constant: match (a.constant, b_constant) {
                                (None, _) => None,
                                (Some(t), None) => Some(t),
                                (Some(l), Some(r)) => Some((l * r)?),
                            }
                            .catch(Self::Int(0)),
                        }
                        .into(),
                    )
                }
            }, // now that all the single-type operations are done, the two sided ones
            (Self::Float(flt), a) => Ok(Self::Float(flt * f64::try_from(a)?)), // get floats out of the way because they're bad
            (a, Self::Float(flt)) => Ok(Self::Float(flt * f64::try_from(a)?)),
            (Self::Symbolic(syc), Self::Int(int)) | (Self::Int(int), Self::Symbolic(syc)) => {
                Ok(Self::Symbolic(
                    //next symbolics because they're specific
                    Symbolic {
                        coeff: syc
                            .coeff
                            .or_merge(|x, y| x * y, Ok(Some(Data::from(int))))?
                            .catch(Self::Int(1)),
                        symbol: syc.symbol,
                        constant: match syc.constant.map(|x| x * int.into()) {
                            None => None,
                            Some(t) => Some(t?),
                        }
                        .catch(Self::Int(0)),
                    }
                    .into(),
                ))
            }
            (Self::Symbolic(syc), Self::Symbol(sym)) | (Self::Symbol(sym), Self::Symbolic(syc)) => {
                Ok(Self::Symbolic(
                    Symbolic {
                        coeff: syc
                            .coeff
                            .or_merge(|x, y| x * y, Ok(Some(Self::Symbol(sym.clone()))))?,
                        symbol: syc.symbol,
                        constant: match syc.constant.map(|x| x * Self::Symbol(sym)) {
                            None => None,
                            Some(t) => Some(t?),
                        },
                    }
                    .into(),
                ))
            }
            (Self::Symbolic(syc), Self::Rational(rat))
            | (Self::Rational(rat), Self::Symbolic(syc)) => Ok(Self::Symbolic(
                Symbolic {
                    coeff: syc
                        .coeff
                        .or_merge(|x, y| x * y, Ok(Some(Self::Rational(rat))))?
                        .catch(Self::Int(1)),
                    symbol: syc.symbol,
                    constant: match syc.constant.map(|x| x * Self::Rational(rat)) {
                        None => None,
                        Some(t) => Some(t?),
                    }
                    .catch(Self::Int(0)),
                }
                .into(),
            )),
            (Self::Symbolic(syc), Self::Radical(rad))
            | (Self::Radical(rad), Self::Symbolic(syc)) => Ok(Self::Symbolic(
                Symbolic {
                    coeff: syc
                        .coeff
                        .or_merge(|x, y| x * y, Ok(Some(Self::Radical(rad.clone()))))?
                        .catch(Self::Int(1)),
                    symbol: syc.symbol,
                    constant: match syc.constant.map(|x| x * Self::Radical(rad)) {
                        None => None,
                        Some(t) => Some(t?),
                    }
                    .catch(Self::Int(0)),
                }
                .into(),
            )),
            (Self::Int(int), Self::Symbol(sym)) | (Self::Symbol(sym), Self::Int(int)) => {
                Ok(Self::Symbolic(
                    Symbolic {
                        coeff: Some(Self::Int(int)),
                        symbol: sym,
                        constant: None,
                    }
                    .into(),
                ))
            }
            (Self::Int(int), Self::Rational(rat)) | (Self::Rational(rat), Self::Int(int)) => {
                let result = rat * int;
                if *result.denom() == 1 {
                    Ok(Self::Int(*result.numer()))
                } else {
                    Ok(Self::Rational(result))
                }
            }
            (Self::Int(int), Self::Radical(rad)) | (Self::Radical(rad), Self::Int(int)) => Ok(
                Self::Radical(Radical::new(rad.coefficient * int, rad.index, rad.radicand)),
            ),
            (Self::Rational(rat), Self::Radical(rad))
            | (Self::Radical(rad), Self::Rational(rat)) => Ok(Self::Radical(Radical::new(
                rad.coefficient * rat,
                rad.index,
                rad.radicand,
            ))),
            (a, Self::Symbol(s)) | (Self::Symbol(s), a) => Ok(Self::Symbolic(
                Symbolic {
                    coeff: Some(a),
                    symbol: s,
                    constant: None,
                }
                .into(),
            )),
        }
    }
}
