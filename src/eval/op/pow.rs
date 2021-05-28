use crate::eval::{op::root::NthRoot, Data, DivisibleBy, Radical, SymbolEval, Symbolic};
use num::rational::Ratio;

pub trait Pow<RHS = Self> {
    type Output;

    fn pow(self, rhs: RHS) -> Self::Output;
}

impl Pow for Data {
    type Output = Result<Self, String>;

    fn pow(self, rhs: Self) -> Self::Output {
        let invert_result = rhs < 0;
        let abs_rhs = if invert_result { -rhs } else { rhs };
        match self {
            Self::Int(i) => match abs_rhs {
                Self::Int(j) => Ok(Data::Int(i.pow(j as u32))),
                Self::Float(f) => Ok(Data::Float((i as f64).powf(f))),
                Self::Radical(r) => Ok(Data::Float((i as f64).powf(r.as_float()))),
                Self::Rational(r) => self
                    .pow(Self::Int(*r.numer()))
                    .and_then(|x| x.nth_root(*r.denom())),
                Self::Symbol(s) => Ok(Self::Float((i as f64).powf(s.symbol_eval()?))),
                Self::Symbolic(s) => Ok(Self::Float((i as f64).powf(s.as_float()))),
            },
            Self::Float(i) => Ok(Self::Float(i.powf(abs_rhs.into()))),
            Self::Rational(i) => {
                Self::Int(*i.numer()).pow(abs_rhs)? / Self::Int(*i.denom()).pow(abs_rhs)?
            }
            Self::Radical(i) => match abs_rhs {
                Self::Int(j) if j.divisible_by(i.index as i64) => Ok(Self::Rational(i.coefficient)
                    .pow(Self::Int(j))?
                    * i.radicand.pow(Self::Int(j / i.index as i64))?),
                Self::Rational(j) => self
                    .pow(Self::Int(*j.numer()))
                    .and_then(|x| x.nth_root(*j.denom())),
                Self::Radical(j) => Ok(Data::Float(i.as_float().powf(j.as_float()))),
                Self::Symbol(j) => Ok(Data::Float(i.as_float().powf(j.symbol_eval()?))),
                Self::Symbolic(j) => Ok(Data::Float(i.as_float().powf(j.as_float()))),
            },
            Self::Symbol(i) => match abs_rhs {
                Self::Int(j) => Ok(Self::Symbol(i).naive_pow(j as u32)),
                Self::Rational(j) => Self::Symbol(i)
                    .pow(Self::Int(*j.numer()))
                    .and_then(|x| x.nth_root(*j.denom())),
                Self::Float(j) => i
                    .symbol_eval()
                    .and_then(|x| Ok(x.powf(j)))
                    .map(|d| Data::from(d)),
                Self::Symbol(j) => i
                    .symbol_eval()
                    .and_then(|x| j.symbol_eval().map(|y| x.powf(y)))
                    .map(|d| Data::from(d)),
                Self::Symbolic(j) => i
                    .symbol_eval()
                    .and_then(|x| Ok(x.powf(j.as_float())))
                    .map(|d| Data::from(d)),
                Self::Radical(j) => i
                    .symbol_eval()
                    .and_then(|x| Ok(x.powf(j.as_float())))
                    .map(|d| Data::from(d)),
            },
            Self::Symbolic(i) => {
                if i.constant == None {
                    match abs_rhs {
                        Self::Int(j) => Ok(Self::Symbolic(
                            Symbolic {
                                coeff: match i.coeff {
                                    None => Some(Self::Symbol(i.symbol)),
                                    Some(n) => Some(n.pow(Self::Int(j))? * Self::Symbol(i.symbol)),
                                },
                                symbol: i.symbol,
                                constant: None,
                            }
                            .into(),
                        )),
                        Self::Rational(j) => Self::Symbolic(i)
                            .pow(Self::Int(*j.numer()))
                            .and_then(|x| x.nth_root(*j.denom())),
                        _ => self.as_float().pow(abs_rhs.as_float()),
                    }
                } else {
                    self.as_float().pow(abs_rhs.as_float())
                }
            }
        }
        .and_then(|k| {
            if invert_result {
                Data::Int(1) / k
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

impl<T> NaivePow for T
where
    T: std::ops::Mul<T, Output = T>,
{
    type Output = Self;
    fn naive_pow(self, pow: u32) -> Self::Output {
        let mut result = self;
        for _ in 0..pow - 1 {
            result = result * self;
        }
        result
    }
}
