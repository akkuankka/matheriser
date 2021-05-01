use crate::eval::{Data, DivisibleBy, OrMerge, Radical, Symbolic};
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
                            .or_merge(Some(k), |x, y| x * y)
                            .or_merge(Some(Self::Symbol(b.symbol)), |x, y| x * y ),
                        symbol: a.symbol,
                        constant: a.constant.map(|x| x * k * b.symbol),
                    }
                    .into(),
                ),
                _ => { // needs testing, IDK if this actually does what I think it does
                    Self::Symbolic(
                        Symbolic {
                            coeff: a
                                .coeff
                                .or_merge(b.coeff, |x, y| x * y)
                                .or_merge(Some(Self::Symbol(b.symbol)), |x, y| x * y),
                            symbol: a.symbol,
                            constant: a.constant.map(|x| x * Data::Symbol(b.symbol) * b.coeff.unwrap_or(Data::Int(1)))
                        }.into()
                    ) +
                    Self::Symbolic (
                        Symbolic {
                            coeff: a
                                .coeff
                                .or_merge(b.constant, |x, y| x * y)
                                ,
                            symbol: a.symbol,
                            constant: a.constant.map(|x| x * b.constant.unwrap_or(Data::Int(1)))
                        }.into()
                    )
                }
            } // now that all the single-type operations are done, the two sided ones
            (Self::Symbolic(syc), Self::Int(int)) => Self::Symbolic(
                Symbolic {
                    coeff: syc.coeff.or_merge(rhs.into(), |x, y| x * y),
                    symbol: syc.symbol,
                    constant: syc.map(|x|, x * int)
                }.into()
            )
        }
    }
}
