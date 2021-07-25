use crate::eval::{op::pow::Pow, Number, DivisibleBy, Radical, SymbolEval, Symbolic};
use num::rational::Ratio;
use std::convert::TryFrom;
use std::ops::Div;

impl Div for Number {
    type Output = Result<Self, String>;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs == Self::Int(0) {
            Err(String::from("Maths error: Divide by 0"))
        } else if rhs == Self::Int(1) {
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
                    Self::Float(m) => Ok(Self::Float(n as f64 / m)),
                    Self::Symbol(m) => Ok(Self::Float(n as f64 / m.symbol_eval()?)),
                    Self::Symbolic(m) => Ok(Self::Float(n as f64 / m.as_float()?)),
                    Self::Radical(r) => {
                        (Self::Int(n) * Self::Radical(r.clone().conjugate()?))? / *r.radicand
                    }
                    Self::Rational(m) => {
                        (Self::Int(n) * Self::Int(*m.denom()))? / Self::Int(*m.numer())
                    }
                },
                Self::Symbol(s) => match rhs {
                    Self::Int(m) => Ok(Self::Symbolic(
                        Symbolic {
                            coeff: Some(Number::Rational(Ratio::from((1, m)))),
                            symbol: s,
                            constant: None,
                        }
                        .into(),
                    )),
                    Self::Symbol(m) => {
                        if s == m {
                            Ok(Self::Int(1))
                        } else {
                            Ok(Self::Float(s.symbol_eval()? / m.symbol_eval()?))
                            // this seems unfortunate but this is usually what you'd want
                        }
                    }
                    Self::Symbolic(m) => Ok(Self::Float(s.symbol_eval()? / m.as_float()?)),
                    Self::Rational(m) => Ok(Self::Symbolic(
                        Symbolic {
                            coeff: Some(Self::Rational(m.recip()).into()),
                            symbol: s,
                            constant: None,
                        }
                        .into(),
                    )),
                    _ => Ok(Self::Float(s.symbol_eval()? / f64::try_from(rhs)?)),
                },
                Self::Symbolic(n) => match rhs {
                    Self::Symbol(m) => {
                        if n.symbol == m
                            && match n.constant.as_ref() {
                                None => true,
                                Some(e) => e.divisible_by(&Self::Symbol(m.clone())),
                            }
                        {
                            n.coeff.unwrap_or(Self::Int(0))
                                + (n.constant.unwrap_or(Self::Int(1)) / Self::Symbol(m))?
                        } else {
                            Ok(Self::Symbolic(
                                Symbolic {
                                    coeff: Some(
                                        (n.coeff.unwrap_or(Number::Int(1))
                                            / Self::Symbol(m.clone()))?,
                                    ),
                                    symbol: n.symbol,
                                    constant: {
                                        let r = n.constant.map(|x| x / Self::Symbol(m.clone()));
                                        match r {
                                            None => None,
                                            Some(Err(e)) => return Err(e),
                                            Some(Ok(v)) => Some(v),
                                        }
                                    },
                                }
                                .into(),
                            ))
                        }
                    }
                    a => Ok(Self::Symbolic(
                        Symbolic {
                            coeff: Some((n.coeff.unwrap_or(Number::Int(1)) / a.clone())?),
                            symbol: n.symbol,
                            constant: {
                                let r = n.constant.map(|x| x / a.clone());
                                match r {
                                    None => None,
                                    Some(Err(e)) => return Err(e),
                                    Some(Ok(v)) => Some(v),
                                }
                            },
                        }
                        .into(),
                    )),
                },
                Self::Radical(n) => match rhs {
                    Self::Int(m) => Ok(Self::Radical(Radical::new(
                        n.coefficient / m,
                        n.index,
                        n.radicand,
                    ))),
                    Self::Radical(m) => {
                        if n.divisible_by(&m) {
                            if n.index == m.index && n.radicand == m.radicand {
                                Ok(Self::Radical(Radical::new(
                                    n.coefficient / m.coefficient,
                                    n.index,
                                    n.radicand,
                                )))
                            } else if n.index.divisible_by(m.index) {
                                let rhs_modified = Radical::new(
                                    m.coefficient,
                                    n.index,
                                    m.radicand
                                        .clone()
                                        .pow(
                                            (Number::from(n.index as i64)
                                                / Number::from(m.index as i64))?,
                                        )?
                                        .into(),
                                );
                                if rhs_modified.radicand == n.radicand {
                                    Ok(Self::Radical(Radical::new(
                                        n.coefficient / rhs_modified.coefficient,
                                        n.index,
                                        n.radicand,
                                    )))
                                } else {
                                    Self::Radical(n).as_float()? / Number::from(m.as_float()?)
                                }
                            } else if m.index.divisible_by(n.index) {
                                let lhs_modified = Radical::new(
                                    n.coefficient,
                                    m.index,
                                    n.radicand
                                        .clone()
                                        .pow(
                                            (Number::from(m.index as i64)
                                                / Number::from(n.index as i64))?,
                                        )?
                                        .into(),
                                );
                                if lhs_modified.radicand == n.radicand {
                                    Ok(Self::Radical(Radical::new(
                                        lhs_modified.coefficient / m.coefficient,
                                        n.index,
                                        n.radicand,
                                    )))
                                } else {
                                    Self::Radical(n).as_float()? / Self::Radical(m).as_float()?
                                }
                            } else {
                                Self::Radical(n).as_float()? / Self::Radical(m).as_float()?
                            }
                        } else {
                            Self::Radical(n).as_float()? / Self::Radical(m).as_float()?
                        }
                    }
                    Self::Rational(m) => Ok(Self::Radical(Radical::new(
                        n.coefficient * m.recip(),
                        n.index,
                        n.radicand,
                    ))),
                    b => Self::Radical(n).as_float()? / b.as_float()?,
                },
                Self::Rational(rat) => match rhs {
                    Self::Int(_) => Self::Int(*rat.numer()) / (Self::Int(*rat.denom()) * rhs)?,
                    Self::Rational(r) => {
                        Self::Int(*rat.numer() * *r.denom()) / Self::Int(*rat.denom() * r.numer())
                    }
                    Self::Radical(r) => {
                        Self::Int(*rat.numer())
                            / ((*r.radicand * Self::Rational(r.coefficient))?
                                * Self::Int(*rat.denom()))?
                    }
                    _ => self.as_float()? / rhs.as_float()?,
                },
                a => a.as_float()? / rhs.as_float()?,
            }
        }
    }
}
