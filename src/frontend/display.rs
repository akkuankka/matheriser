/*! These are all the display implementations for `Data`*/

use crate::eval::{radical::Radical, Number, Symbolic, Symbol, op::Mul};
use std::fmt;
use std::fmt::{Display, Formatter};

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            Number::Int(a) => write!(f, "{}", a),
            Number::Float(a) => {
                let maybe_scientific =
                    if format!("{:.8}", a).matches('0').collect::<Vec<_>>().len() >= 4 {
                        format!("{:.8e}", a)
                    } else {
                        format!("{:.8}", a)
                    };
                write!(f, "{}", maybe_scientific)
            }
            Number::Rational(a) => write!(f, "{}/{}", a.numer(), a.denom()),
            Number::Radical(a) => {
                let coeff = if a.coefficient == 1.into() {
                    "".to_string()
                } else if *a.coefficient.denom() == 1 {
                    a.coefficient.numer().to_string()
                } else {
                    format!("{}", Number::Rational(a.coefficient))
                };
                let index = a.index;
                let index_str = if index == 2 {
                    "".to_string()
                } else {
                    index.to_string()
                };
                write!(f, "({}) {}√({})", coeff, index_str, a.radicand)
            }
            Number::Symbol(a) => write!(f, "{}", a.as_utf8()),
            Number::Symbolic(a) => write!(f, "{}", a),
        }
    }
}

trait AsUtf8 {
    fn as_utf8(&self) -> String;
}

impl AsUtf8 for Symbol {
    fn as_utf8(&self) -> String {
        match self {
            Symbol::Pi => "π",
            Symbol::E => "e",
            Symbol::Phi => "ϕ",
            Symbol::Sqrt2 => "√2",
            l => "?",
        }
        .to_string()
    }
}

/// A Data-Exponent pair such as (√3)^2
#[derive(PartialEq, PartialOrd)]
struct DFactor {
    val: Number,
    exponent: u32,
}

impl From<Number> for DFactor {
    fn from(data: Number) -> Self {
        DFactor {
            val: data,
            exponent: 1,
        }
    }
}

use std::collections::HashMap;
use std::rc::Rc;

struct FactorChain {
    symbol_map: HashMap<String, u32>,
    data_factors: Vec<DFactor>,
}

/// this function primarily exists because I cannot be bothered writing a trait
fn insert_or_inc_factor(factors: &mut Vec<DFactor>, insert: Number) {
    let is_one = insert == Number::Int(1);
    let factor_to_add: DFactor = insert.into();
    if is_one {
        if !factors.contains(&DFactor {
            val: Number::from(1),
            exponent: 1,
        }) {
            factors.push(DFactor {
                val: Number::from(1),
                exponent: 1,
            })
        }
    } else if let Ok(i) =
        factors.binary_search_by(|factor| match factor.partial_cmp(&factor_to_add) {
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
    fn new() -> Self {
        FactorChain {
            symbol_map: HashMap::new(),
            data_factors: Vec::new(),
        }
    }

    fn add(&mut self, data: Number) {
        match data {
            Number::Int(_) | Number::Float(_) | Number::Rational(_) => {
                insert_or_inc_factor(&mut self.data_factors, data)
            }
            Number::Symbol(s) => insert_or_inc_symbol(&mut self.symbol_map, s.as_utf8()),
            Number::Radical(rad) => {
                let coeff = Number::Rational(rad.coefficient);
                let rest = Number::Radical(Rc::new(Radical {
                    coefficient: 1.into(),
                    index: rad.index,
                    radicand: rad.radicand.clone()
                }));
                insert_or_inc_factor(&mut self.data_factors, coeff);
                insert_or_inc_factor(&mut self.data_factors, rest);
            }
            Number::Symbolic(s) => {
                if let None = s.constant {
                    insert_or_inc_symbol(&mut self.symbol_map, s.symbol.as_utf8());
                    insert_or_inc_factor(&mut self.data_factors, s.as_ref().coeff.clone().unwrap_or(Number::Int(1)))
                } else {
                    insert_or_inc_factor(&mut self.data_factors, Number::Symbolic(s))
                }
            }
        }
    }

    fn condense_linears(&mut self) {
        let simple_linears_extracted = self
            .data_factors
            .drain_filter(|factor| factor.exponent == 1)
            .map(|x| x.val)
            .reduce(|x, y| x.mul(&y).unwrap_or(Number::from(0)));
        if let Some(linears) = simple_linears_extracted {
            self.data_factors.push(DFactor {
                exponent: 1,
                val: linears,
            })
        }
    }
}

impl Display for Symbolic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut l_factor_chain = FactorChain::new();
        l_factor_chain.add(Number::from(self.symbol));
        let mut l_working = self.coeff.clone().unwrap_or(Number::Int(1));
        l_factor_chain.add(Number::Int(1)); // 1 is implicit here
        loop {
            if let Number::Symbolic(a) = l_working {
                if a.constant == None {
                    l_working = a.coeff.clone().unwrap_or(Number::Int(1));
                    l_factor_chain.add(Number::Symbol(a.symbol));
                    continue;
                } else {
                    l_factor_chain.add(Number::Symbolic(a));
                    break;
                }
            } else {
                l_factor_chain.add(l_working);
                break;
            }
        }
        l_factor_chain.data_factors = l_factor_chain
            .data_factors
            .into_iter()
            .filter(|x| {
                x != &DFactor {
                    val: Number::Int(1),
                    exponent: 1,
                }
            })
            .collect();
        let mut r_factor_chain = FactorChain::new();
        if let Some(a) = self.constant.clone() {
            let mut r_working = a;
            loop {
                if let Number::Symbolic(a) = r_working {
                    if a.constant == None {
                        r_working = a.coeff.clone().unwrap_or(Number::Int(1));
                        r_factor_chain.add(Number::Symbol(a.symbol));
                        continue;
                    } else {
                        r_factor_chain.add(Number::Symbolic(a));
                        break;
                    }
                } else {
                    r_factor_chain.add(r_working);
                    break;
                }
            }
        }
        r_factor_chain.condense_linears();
        l_factor_chain.condense_linears();
        let mut l_symbol_factors: Vec<(String, u32)> = l_factor_chain
            .symbol_map
            .into_iter()
            .collect::<Vec<(String, u32)>>(); // sort all of the symbol factors
        l_symbol_factors.sort_unstable_by(|(a, _), (b, _)| a.cmp(&b));
        let mut r_symbol_factors: Vec<(String, u32)> = r_factor_chain
            .symbol_map
            .into_iter()
            .collect::<Vec<(String, u32)>>(); // sort all of the symbol factors
        r_symbol_factors.sort_unstable_by(|(a, _), (b, _)| a.cmp(&b));

        fn stringify_symbol_chain(input: &[(String, u32)]) -> String {
            let mut output = String::new();
            if input.len() == 1 {
                if input[0].1 == 1 {
                    format!("{}", input[0].0)
                } else {
                    format!("{}^{}", input[0].0, input[0].1)
                }
            } else {
                for (symb, exp) in input.iter().rev() {
                    if *exp == 1 {
                        output.push_str(&symb)
                    } else {
                        output.push_str(format!("({}^{})", symb, exp).as_str())
                    }
                }
                output
            }
        }
        let l_symbols = stringify_symbol_chain(&l_symbol_factors); // as a string
        let r_symbols = stringify_symbol_chain(&r_symbol_factors);

        fn stringify_factor_chain(input: &[DFactor]) -> String {
            if input.len() == 1 {
                format!("{}", input[0].val)
            } else {
                let mut output = String::new();
                for i in input.iter().rev() {
                    if Some(i) == input.last() {
                        output.push_str(format!("{} × ", i.val).as_str())
                    } else if Some(i) == input.first() {
                        output.push_str(format!("({})^{}", i.val, i.exponent).as_str())
                    } else {
                        output.push_str(format!("({})^{} × ", i.val, i.exponent).as_str())
                    }
                }
                output
            }
        }
        let l_factors = stringify_factor_chain(&l_factor_chain.data_factors);
        let r_factors = stringify_factor_chain(&r_factor_chain.data_factors);
        if r_factors == "" && r_symbols == "" {
            write!(f, "{}{}", l_factors, l_symbols)
        } else {
            write!(f, "{}{} + {}{}", l_factors, l_symbols, r_factors, r_symbols)
        }
    }
}

