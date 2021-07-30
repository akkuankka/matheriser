use super::{Div, Mul};
use crate::eval::{op::root::NthRoot, DivisibleBy, Number, SymbolEval, Symbolic};
use std::convert::TryInto;

pub trait Pow<RHS = Self> {
    type Output;

    fn pow(&self, rhs: &RHS) -> Self::Output;
}

impl Pow for Number {
    type Output = Result<Self, String>;

    fn pow(&self, rhs: &Self) -> Self::Output {
        let invert_result = *rhs < Number::from(0);
        let abs_rhs = if invert_result {
            -rhs
        } else {
            rhs.clone()
        };
        match self.clone() {
            Self::Int(i) => match abs_rhs {
                Self::Int(j) => Ok(Number::Int(i.pow(j as u32))),
                Self::Float(f) => Ok(Number::Float((i as f64).powf(f))),
                Self::Radical(r) => Ok(Number::Float((i as f64).powf(r.as_float()?))),
                Self::Rational(r) => self
                    .pow(&Self::Int(*r.numer()))
                    .and_then(|x| x.nth_root(*r.denom())),
                Self::Symbol(s) => Ok(Self::Float((i as f64).powf(s.symbol_eval()?))),
                Self::Symbolic(s) => Ok(Self::Float((i as f64).powf(s.as_float()?))),
            },
            Self::Float(i) => Ok(Self::Float(i.powf(abs_rhs.try_into()?))),
            Self::Rational(i) => (&Self::Int(*i.numer())
                .pow(&abs_rhs)?)
                .div(&Self::Int(*i.denom()).pow(&abs_rhs)?),
            Self::Radical(i) => match abs_rhs {
                Self::Int(j) if j.divisible_by(i.index as i64) => Self::Rational(i.coefficient)
                    .pow(&Self::Int(j))?
                    .mul(&i.radicand.pow(&Self::Int(j / i.index as i64))?),
                Self::Rational(j) => match Self::Radical(i).pow(&Self::Int(*j.numer())) {
                    Ok(x) => x.nth_root(*j.denom()),
                    Err(e) => Err(e),
                },
                Self::Radical(j) => Ok(Number::Float(i.as_float()?.powf(j.as_float()?))),
                Self::Symbol(j) => Ok(Number::Float(i.as_float()?.powf(j.symbol_eval()?))),
                Self::Symbolic(j) => Ok(Number::Float(i.as_float()?.powf(j.as_float()?))),
                a => Ok(Number::Float(i.as_float()?.powf(a.try_into()?))),
            },
            Self::Symbol(i) => match abs_rhs {
                Self::Int(j) => Self::Symbol(i).naive_pow(j as u32),
                Self::Rational(j) => Self::Symbol(i)
                    .pow(&Self::Int(*j.numer()))
                    .and_then(|x| x.nth_root(*j.denom())),
                Self::Float(j) => i
                    .symbol_eval()
                    .and_then(|x| Ok(x.powf(j)))
                    .map(|d| Number::from(d)),
                Self::Symbol(j) => i
                    .symbol_eval()
                    .and_then(|x| j.symbol_eval().map(|y| x.powf(y)))
                    .map(|d| Number::from(d)),
                Self::Symbolic(j) => i
                    .symbol_eval()
                    .and_then(|x| Ok(x.powf(j.as_float()?)))
                    .map(|d| Number::from(d)),
                Self::Radical(j) => i
                    .symbol_eval()
                    .and_then(|x| Ok(x.powf(j.as_float()?)))
                    .map(|d| Number::from(d)),
            },
            Self::Symbolic(i) => {
                if i.constant == None {
                    match abs_rhs {
                        Self::Int(j) => Ok(Self::Symbolic(
                            Symbolic {
                                coeff: match i.coeff {
                                    None => Some(Self::Symbol(i.symbol.clone())),
                                    Some(ref n) => Some(
                                        n.pow(&Self::Int(j))?
                                            .mul(&Self::Symbol(i.symbol.clone()))?,
                                    ),
                                },
                                symbol: i.symbol,
                                constant: None,
                            }
                            .into(),
                        )),
                        Self::Rational(j) => Self::Symbolic(i)
                            .pow(&Self::Int(*j.numer()))
                            .and_then(|x| x.nth_root(*j.denom())),
                        _ => Self::Symbolic(i).as_float()?.pow(&abs_rhs.as_float()?),
                    }
                } else {
                    Self::Symbolic(i).as_float()?.pow(&abs_rhs.as_float()?)
                }
            }
        }
        .and_then(|k| {
            if invert_result {
                Number::Int(1).div(&k)
            } else {
                Ok(k)
            }
        })
    }
}

trait NaivePow {
    type Output;
    fn naive_pow(self, pow: u32) -> Self::Output;
}

impl<T, E> NaivePow for T
where
    T: Mul<Output = Result<T, E>> + Clone,
{
    type Output = Result<T, E>;
    fn naive_pow(self, pow: u32) -> Self::Output {
        let mut result = self.clone();
        for _ in 0..pow - 1 {
            result = (result.mul(&self.clone()))?;
        }
        Ok(result)
    }
}
