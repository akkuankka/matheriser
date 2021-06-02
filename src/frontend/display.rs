/*! These are all the display implementations for `Data`*/

use crate::eval::{radical::Radical, Data, Symbolic};
use std::fmt;
use std::fmt::{Display, Formatter};

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Data::Int(a) => write!(f, "{}", a),
            Data::Float(a) => {
                let maybe_scientific =
                    if format!("{:.8}", a).matches('0').collect::<Vec<_>>().len() >= 4 {
                        format!("{:.8e}", a)
                    } else {
                        format!("{:.8}", a)
                    };
                write!(f, "{}", maybe_scientific)
            }
            Data::Rational(a) => write!(f, "{}/{}", a.numer(), a.denom()),
            Data::Radical(a) => {
                let coeff = if a.coefficient == 1.into() {
                    "".to_string()
                } else if *a.coefficient.denom() == 1 {
                    a.coefficient.numer().to_string()
                } else {
                    format!("{}", Data::Rational(a.coefficient))
                };
                let index = if a.index == 2 {
                    ""
                } else {
                    format!("`{}`", a.index.to_string())
                };
                write!(f, "({}) {}√({})", coeff, index, a.radicand)
            }
            Data::Symbol(a) => write!(f, "{}", a.as_utf8()),
            Data::Symbolic(a) => write!(f, "{}", a),
        }
    }
}

trait AsUtf8 {
    fn as_utf8(&self) -> String;
}

impl AsUtf8 for String {
    fn as_utf8(&self) -> String {
        match self.as_str() {
            "pi" | "Pi" => "π",
            "e" | "E" => "e",
            "phi" | "Phi" => "ϕ",
            "sqrt2" | "root2" => "√",
            l => l,
        }
        .to_string()
    }
}

/// A Data-Exponent pair such as (√3)^2
#[derive(PartialEq, PartialOrd)]
struct DFactor {
    val: Data,
    exponent: u32,
}

impl From<Data> for DFactor {
    fn from(data: Data) -> Self {
        DFactor {
            val: data,
            exponent: 1,
        }
    }
}

use std::collections::HashMap;

struct FactorChain {
    symbol_map: HashMap<String, u32>,
    data_factors: Vec<DFactor>,
}

/// this function primarily exists because I cannot be bothered writing a trait
fn insert_or_inc_factor(factors: &mut Vec<DFactor>, insert: Data) {
    let factor_to_add: DFactor = insert.into();
    if insert == Data::Int(1) {
    } else if let Ok(i) =
        factors.binary_search_by(|&factor| match factor.partial_cmp(&factor_to_add) {
            Some(ord) => ord,
            None => std::cmp::Ordering::Less,
        })
    {
        factors.push(factor_to_add);
        factors.swap_remove(i);
    } else {
        factors.push(factor_to_add);
    }
}

fn insert_or_inc_symbol(symbols: &mut HashMap<String, u32>, s: String) {
    if symbols.contains_key(&s) {
        if let Some(val) = symbols.get_mut(&s) {
            *val += 1;
        }
    } else {
        symbols.insert(s, 1);
    }
}

impl FactorChain {
    fn add(&mut self, data: Data) {
        match data {
            Data::Int(_) | Data::Float(_) | Data::Rational(_) => {
                insert_or_inc_factor(&mut self.data_factors, data)
            }
            Data::Symbol(s) => insert_or_inc_symbol(&mut self.symbol_map, s),
            Data::Radical(rad) => {
                let coeff = Data::Rational(rad.coefficient);
                let rest = Data::Radical(Radical {
                    coefficient: 1.into(),
                    ..rad
                });
                insert_or_inc_factor(&mut self.data_factors, coeff);
                insert_or_inc_factor(&mut self.data_factors, rest);
            }
            Data::Symbolic(s) => {
                if let None = s.constant {
                    insert_or_inc_symbol(&mut self.symbol_map, s.symbol);
                    insert_or_inc_factor(&mut self.data_factors, s.coeff.unwrap_or(Data::Int(1)))
                } else {
                    insert_or_inc_factor(&mut self.data_factors, Data::Symbolic(s))
                }
            }
        }
    }
}
