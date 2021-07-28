use crate::{
    parser::{BinaryOp, ExprTree, UnaryOp},
    util::option::{Catch, OrMerge},
    eval::op::calculate_fn::CalculateFn
};
use num::rational::Ratio;
use op::pow::Pow;
use radical::Radical;
use std::convert::{TryFrom, TryInto};
use std::ops::{Mul, Rem};
use std::rc::Rc;

mod op;
mod ord;
pub mod radical;

/// This is a symbolic expression, not like the ones in lisp,
/// these are for dealing with symbolic numbers like pi and e
#[derive(Clone, PartialEq)]
pub struct Symbolic {
    pub coeff: Option<Number>,
    pub symbol: Symbol,
    pub constant: Option<Number>,
}

impl DivisibleBy<&Number> for Symbolic {
    fn divisible_by(&self, rhs: &Number) -> bool {
        if let Number::Symbol(s) = rhs {
            if self.constant == None && &self.symbol == s {
                return true;
            }
        }
        match &self.coeff {
            Some(d) => {
                d.divisible_by(rhs)
                    && match &self.constant {
                        Some(d) => d.divisible_by(rhs),
                        None => true,
                    }
            }
            _ => false,
        }
  }
}

impl DivisibleBy<u16> for Symbolic {
    fn divisible_by(&self, rhs: u16) -> bool {
        match &self.coeff {
            Some(d) => {
                d.divisible_by(rhs)
                    && match &self.constant {
                        Some(d) => d.divisible_by(rhs),
                        None => true,
                    }
            }
            _ => false,
        }
    }
}

impl Symbolic {
    fn as_float(self) -> Result<f64, String> {
        Ok(
            f64::try_from(self.coeff.unwrap_or(Number::Int(1)))? * self.symbol.symbol_eval()?
                + match self.constant {
                    Some(d) => d.try_into()?,
                    None => 0.,
                },
        )
    }
    /// Makes sure symbolics aren't illformed or symbols in disguise, returns Err(Symbol) if they are
    pub fn sanity_check(self) -> Result<Self, Symbol> {
        let result = Self {
            coeff: self.coeff.catch(Number::Int(1)),
            symbol: self.symbol,
            constant: self.constant.catch(Number::Int(0)),
        };
        if result.coeff == None && result.constant == None {
            Err(result.symbol)
        } else {
            Ok(result)
        }
    }
}

/// The basic data type that all our calculations act on, yes this is very large
/// for what might be in other implementations a `f64` but in order to preserve
/// rationals, radicals, and symbols, this needs to be kept.
#[derive(Clone, PartialEq)]
pub enum Number {
    /// a whole number
    Int(i64),
    /// a fraction, we use this to avoid floats
    Rational(Ratio<i64>),
    /// a fancy form of a square root, see the module itself `radical`
    Radical(Rc<Radical>),
    /// pi, e, etc, the contents may eventually become a `non_exhaustive` enum to save space
    Symbol(Symbol),
    /// these are bad and we try and avoid them, because of precision errors they
    /// tend to infect any numbers they come into contact with.
    //  In future, this may become a floating-point decimal type, to avoid some of that,
    //  if performance is not an issue
    Float(f64),
    /// see the documentation for `Symbolic`
    Symbolic(Rc<Symbolic>),
}

impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Self::Int(n)
    }
}

impl From<Symbol> for Number {
    fn from(s: Symbol) -> Self {
        Self::Symbol(s)
    }
}

impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Self::Float(n)
    }
}

impl Number {
    ///flattens any Data value down to a f64
    /// once float-land has been entered, there are only a few cases where we can get out of it.

    fn as_float(self) -> Result<Self, String> {
        Ok(match self {
            Self::Float(_) => self,
            Self::Int(n) => Self::Float(n as f64),
            Self::Rational(n) => Self::Float(ratio_as_float(*n)),
            Self::Symbol(s) => Self::Float(s.symbol_eval().unwrap_or(0.)),
            Self::Symbolic(s) => Self::Float(s.as_float()?),
            Self::Radical(r) => Self::Float(r.as_float()?),
        })
    }
}

fn ratio_as_float(r: Ratio<i64>) -> f64 {
    let (numer, denom) = r.into();
    numer as f64 / denom as f64
}

#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq)]
enum Symbol {
    Pi, E, Phi, Sqrt2
}

///This trait describes the behaviour of a stringy symbol turning into a number

trait SymbolEval {
    fn symbol_eval(&self) -> Result<f64, String>;
}

impl SymbolEval for String {
    fn symbol_eval(&self) -> Result<f64, String> {
        Ok(match self.as_str() {
            "pi" | "Pi" => std::f64::consts::PI,
            "e" | "E" => std::f64::consts::E,
            "phi" | "Phi" => 1.61803398874989484820458683436563811,
            "sqrt2" | "root2" => std::f64::consts::SQRT_2,
            e => return Err(format!("constant {} not recognised", e.to_string())),
        })
    }
}

impl SymbolEval for Symbol {
    fn symbol_eval(&self) -> Result<f64, String> {
        Ok(match self {
            Pi => std::f64::consts::PI,
            E => std::f64::consts::E,
            Phi => 1.61803398874989484820458683436563811,
            Sqrt2 => std::f64::consts::SQRT_2,
            _ => return Err("constant not recognised !".to_string())
        })
    }
}

impl TryFrom<String> for Symbol {
    type Error = String;
    fn try_from(s: String) -> Result<Symbol, Self::Error> {
        Ok(match s.as_str() {
            "pi" | "Pi" => Symbol::Pi,
            "e" | "E" => Symbol::E,
            "phi" | "Phi" => Symbol::Phi,
            "sqrt2" | "root2" => Symbol::Sqrt2,
            e => return Err(format!("constant {} not recognised", e.to_string())),
        })
    }
}

/// This trait allows us to wrap a calculation for if something is divisible by something else,
/// which is useful generically for reducing radicals, rationals and symbolic expressions *not the lisp sort*
trait DivisibleBy<T> {
    fn divisible_by(&self, divisor: T) -> bool;
}

impl<T, U> DivisibleBy<T> for U
where
    U: Rem<T> + GenericThunk + Copy,
    <U as Rem<T>>::Output: PartialEq + From<u8>,
{
    fn divisible_by(&self, divisor: T) -> bool {
        *self % divisor == <U as Rem<T>>::Output::from(0)
    }
}

trait GenericThunk {}

impl GenericThunk for i64 {}
impl GenericThunk for u16 {}
impl GenericThunk for u32 {}
impl GenericThunk for f64 {}

impl<T> !GenericThunk for Ratio<T> {}
impl !GenericThunk for Radical {}

impl<T> DivisibleBy<Self> for Ratio<T>
where
    T: DivisibleBy<T> + Mul<Output = T> + Copy,
{
    fn divisible_by(&self, rhs: Self) -> bool {
        // a/b is divisible by c/d when d is divisible by bc:
        // a/b / c/d = a/b * d/c = ad/bc => a(k | k e Z)
        rhs.denom().divisible_by(*self.denom() * *rhs.numer())
    }
}

impl DivisibleBy<u16> for Number {
    fn divisible_by(&self, divisor: u16) -> bool {
        match &self {
            Self::Int(n) => n.divisible_by(divisor as i64),
            Self::Radical(rad) => rad.divisible_by(divisor),
            Self::Rational(rat) => rat.divisible_by(Ratio::from(divisor as i64)),
            Self::Symbolic(s) => s.divisible_by(divisor),
            _ => false,
        }
    }
}

impl DivisibleBy<&Number> for Number {
    fn divisible_by(&self, divisor: &Number) -> bool {
        match &self {
            //let's get the easy cases out of the way:
            // a symbol is only divisible by itself (or maybe an illformed symbolic but that's already an error)
            Self::Symbol(s) => match &divisor {
                Self::Symbol(t) if s == t => true,
                _ => false,
            },
            // it's already implemented for Symbolic
            Self::Symbolic(s) => s.divisible_by(divisor),
            // ints and bigints are also only divisible by integers
            Self::Int(n) => match &divisor {
                Self::Int(m) => n.divisible_by(m),
                _ => false,
            },
            // Rationals: within our Data enum, Rationals should not be integers in disguise, that should get caught by the reduction step, which means that the implementation provided by the generic above is fine
            Self::Rational(n) => match &divisor {
                Self::Rational(m) => n.divisible_by(*m),
                _ => false,
            },
            // Radicals are a bit tricky
            Self::Radical(n) => match &divisor {
                Self::Int(m) => n.divisible_by(*m),
                Self::Rational(rad) => n.divisible_by(rad), 
                Self::Radical(m) => n.divisible_by(&**m), 
                _ => false,
            },
            Self::Float(n) => match divisor {
                Self::Float(m) => n.divisible_by(m),
                _ => false,
            },
        }
    }
}

impl TryFrom<Number> for f64 {
    type Error = String;

    fn try_from(d: Number) -> Result<Self, Self::Error> {
        let f_d = d.as_float()?;
        if let Number::Float(n) = f_d {
            return Ok(n);
        } else {
            unreachable!()
        }
    }
}

// this is the bit that actually does the maths
impl ExprTree {
    pub fn eval(self) -> Result<Number, String> {
        match self {
            ExprTree::Val(k) => Ok(k),
            ExprTree::UNode(op, t) => match op {
                UnaryOp::Neg => t.eval().map(|x| -x),
                UnaryOp::Word(w) => t.eval().and_then(|x| x.calculate_fn(&w))
            },
            ExprTree::BNode(op, lhs, rhs) => {
                let l = lhs.eval()?;
                let r = rhs.eval()?;
                match op {
                    BinaryOp::Plus => l.add(&r),
                    BinaryOp::Minus => l - r,
                    BinaryOp::Mul => l.mul(&r),
                    BinaryOp::Exp => l.pow(r),
                    BinaryOp::Div => l / r,
                }
            }
        }
    }
}
