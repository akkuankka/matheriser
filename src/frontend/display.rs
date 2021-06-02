/*! These are all the display implementations for `Data`*/

use crate::eval::{Symbolic, Data, radical::Radical};
use std::fmt::{Display, Formatter};
use std::fmt;

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Data::Int(a) => write!(f, "{}", a),
            Data::Float(a) => {
                let maybe_scientific = if format!("{:.8}", a).matches('0').collect::<Vec<_>>().len() >= 4 {
                    format!("{:.8e}", a)
                } else {
                   format!("{:.8}", a)
                };
                write!(f, "{}", maybe_scientific)
            }
            Data::Rational(a) => write!(f, "{}/{}",a.numer(), a.denom()),
            Data::Radical(a) => {
                let coeff = if *a.coefficient.denom() == 1 {
                    a.coefficient.numer().to_string()
                }
                else {
                    format!("{}", Data::Rational(a.coefficient))
                };
                let index = if a.index == 2 {""} else {format!("`{}`", a.index.to_string())};
                write!(f, "({}) {}√({})", coeff, index, a.radicand)
            }
            Data::Symbol(a) => write!(f, "{}", a.as_utf8()),
            Data::Symbolic(a) => write!(f, "{}", a)
        }
    }
}


