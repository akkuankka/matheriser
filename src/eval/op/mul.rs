use crate::eval::{Data, DivisibleBy, Radical, Symbolic};
use crate::util::option::{Catch, OrMerge};
use std::ops::Mul;

impl Mul for Data {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            // Easy ones first: same type so we get commutation free
            (Self::Int(a), Self::Int(b)) => Self::Int(a * b),
            (Self::Float(a), Self::Float(b)) => Self::Float(a * b),
            // We can't rely entirely on the implementation that num provides, because when the denominator = 1 it needs to be an Int
            (Self::Rational(a), Self::Rational(b)) => {
                let result = a * b;
                if *result.denom() == 1 {
                    Self::Int(*result.numer())
                } else {
                    Self::Rational(result)
                }
            }
            // Ok now we have to implement this ourself
            (Self::Radical(a), Self::Radical(b)) => {
                if a.index == b.index {
                    Self::Radical(Radical::new(
                        a.coefficient * b.coefficient,
                        a.index,
                        Box::new(*a.radicand * *b.radicand),
                    ))
                } else if a.index.divisible_by(b.index) {
                    Self::Radical(Radical::new(
                        a.coefficient * b.coefficient,
                        a.index,
                        Box::new(*a.radicand * b.radicand.pow(a.index / b.index)),
                    ))
                } else if b.index.divisible_by(a.index) {
                    Self::Radical(Radical::new(
                        b.coefficient * a.coefficient,
                        b.index,
                        Box::new(*b.radicand * a.radicand.pow(b.index / a.index)),
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
                    Self::Float(a.as_float() * b.as_float())
                }
            }
            (Self::Symbol(a), Self::Symbol(b)) => Self::Symbolic(
                Symbolic {
                    coeff: Some(Self::Symbol(a)),
                    symbol: b,
                    constant: None,
                }
                .into(),
            ),
            (Self::Symbolic(a), Self::Symbolic(b)) => match (b.coeff, b.constant) {
                (None, None) => {
                    println!("This is embarassing, I messed up some algebra, it's fine though");
                    Self::Symbolic(
                        Symbolic {
                            coeff: Some(Self::Symbol(a.symbol)),
                            symbol: b.symbol,
                            constant: None,
                        }
                        .into(),
                    )
                }
                (Some(k), None) => Self::Symbolic(
                    Symbolic {
                        coeff: a
                            .coeff
                            .or_merge(|x, y| x * y, Some(k))
                            .or_merge(|x, y| x * y, Some(Self::Symbol(b.symbol))),
                        symbol: a.symbol,
                        constant: a.constant.map(|x| x * k * b.symbol.into()),
                    }
                    .into(),
                ),
                _ => {
                    // needs testing, IDK if this actually does what I think it does
                    Self::Symbolic(
                        Symbolic {
                            coeff: a
                                .coeff
                                .or_merge(|x, y| x * y, b.coeff)
                                .or_merge(|x, y| x * y, b.symbol.into())
                                .catch(Self::Int(1)),
                            symbol: a.symbol,
                            constant: a
                                .constant
                                .map(|x| x * b.symbol.into()) // if it is Some(x), multiply it by symbol
                                .and_then(|x| b.coeff.or_merge(|a, b| a * b, x)) // if it is Some(x), multiply it by coefficient
                                .catch(Self::Int(0)),
                        }
                        .into(),
                    ) + Self::Symbolic(
                        Symbolic {
                            coeff: a
                                .coeff
                                .or_merge(|x, y| x * y, b.constant)
                                .catch(Self::Int(1)),
                            symbol: a.symbol,
                            constant: a
                                .constant
                                .and_then(|x| b.constant.or_merge(|a, b| a * b, x))
                                .catch(Self::Int(0)),
                        }
                        .into(),
                    )
                }
            }, // now that all the single-type operations are done, the two sided ones
            (Self::Float(flt), _) => Self::Float(flt * rhs.into()), // get floats out of the way because they're bad
            (_, Self::Float(flt)) => Self::Float(flt * self.into()),
            (Self::Symbolic(syc), Self::Int(int)) | (Self::Int(int), Self::Symbolic(syc)) => {
                Self::Symbolic(
                    //next symbolics because they're specific
                    Symbolic {
                        coeff: syc.coeff.or_merge(|x, y| x * y, rhs).catch(Self::Int(1)),
                        symbol: syc.symbol,
                        constant: syc.constant.map(|x| x * int.into()).catch(Self::Int(0)),
                    }
                    .into(),
                )
            }
            (Self::Symbolic(syc), Self::Symbol(sym)) | (Self::Symbol(sym), Self::Symbolic(syc)) => {
                Self::Symbolic(
                    Symbolic {
                        coeff: syc.coeff.or_merge(|x, y| x * y, Self::Symbol(sym)),
                        symbol: syc.symbol,
                        constant: syc.constant.map(|x| x * Self::Symbol(sym)),
                    }
                    .into(),
                )
            }
            (Self::Symbolic(syc), Self::Rational(rat))
            | (Self::Rational(rat), Self::Symbolic(syc)) => Self::Symbolic(
                Symbolic {
                    coeff: syc
                        .coeff
                        .or_merge(|x, y| x * y, Self::Rational(rat))
                        .catch(Self::Int(1)),
                    symbol: syc.symbol,
                    constant: syc
                        .constant
                        .map(|x| x * Self::Rational(rat))
                        .catch(Self::Int(0)),
                }
                .into(),
            ),
            (Self::Symbolic(syc), Self::Radical(rad))
            | (Self::Radical(rad), Self::Symbolic(syc)) => Self::Symbolic(
                Symbolic {
                    coeff: syc
                        .coeff
                        .or_merge(|x, y| x * y, Self::Radical(rad))
                        .catch(Self::Int(1)),
                    symbol: syc.symbol,
                    constant: syc
                        .constant
                        .map(|x| x * Self::Radical(rad))
                        .catch(Self::Int(0)),
                }
                .into(),
            ),
            (Self::Int(int), Self::Symbol(sym)) | (Self::Symbol(sym), Self::Int(int)) => {
                Self::Symbolic(
                    Symbolic {
                        coeff: Some(Self::Int(int)),
                        symbol: sym,
                        constant: None,
                    }
                    .into(),
                )
            }
            (Self::Int(int), Self::Rational(rat)) | (Self::Rational(rat), Self::Int(int)) => {
                let result = rat * int;
                if *result.denom() == 1 {
                    Self::Int(*result.numer())
                } else {
                    Self::Rational(result)
                }
            }
            (Self::Int(int), Self::Radical(rad)) | (Self::Radical(rad), Self::Int(int)) => {
                Self::Radical(Radical::new(rad.coefficient * int, rad.index, rad.radicand))
            }
            (Self::Rational(rat), Self::Radical(rad))
            | (Self::Radical(rad), Self::Rational(rat)) => {
                Self::Radical(Radical::new(rad.coefficient * rat, rad.index, rad.radicand))
            }
        }
    }
}
