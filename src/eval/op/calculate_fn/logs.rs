use crate::eval::{Number, Symbolic, op::{Add, Sub}, Symbol};


type DataResult = Result<Number, String>;

pub fn log_10(x: Number) -> DataResult {
    if x <= 0.into() {
        // x is positive
        return Err("Error: Logarithm of a non-positive number".to_string());
    }
    match x {
        Number::Int(n) => {
            if n % 10 == 0 {
                Ok(Number::from((n as f64).log10() as i64)) // cast is sane because this should be a whole number
            } else {
                Ok(Number::from((n as f64).log10()))
            }
        }
        Number::Float(n) => Ok(Number::Float(n.log10())),
        Number::Rational(r) => {
            // this is an optimisation for precision rather than speed, any cases where this would result in anything other than a float would get cancelled out anyway, but the more we can avoid floating-poing errors the better
            let (numer, denom) = (*r.numer(), *r.denom());
            log_10(Number::from(numer))?.sub(&log_10(Number::from(denom))?)
        }
        otherwise => log_10(otherwise.as_float()?)
    }
}

pub fn natural_log(x: Number) -> DataResult {
    if x <= 0.into() {
        return Err("Error: Logarithm of a non-positive number".to_string())
    }
    match x {
        Number::Symbol(Symbol::E) => Ok(Number::Int(1)),
        Number::Symbolic(s) => {
            let Symbolic {ref coeff, symbol, ref constant} = *s;
            if symbol == Symbol::E {
                if let (Some(c), None) = (&coeff, &constant) {
                    if let Some(n) = recursive_e_count(&c) {
                        Ok(Number::Int(n + 1))
                    } else {
                        natural_log(c.clone()).and_then(|x| x.add(&Number::from(1)))
                    }
                } else if constant.is_none() && coeff.is_none() {
                    Ok(Number::Int(1))
                } else {
                    Symbolic {coeff: coeff.clone(), symbol, constant: constant.clone()}.as_float().map(|x| Number::from(x.ln()))
                }
            } else {
                Symbolic {coeff: coeff.clone(), symbol, constant: constant.clone()}.as_float().map(|x| Number::from(x.ln()))
            } 
        }
        Number::Radical(r) => {
            if r.coefficient == 1.into() && r.radicand == Number::Symbol(Symbol::E) {
                Ok(Number::Rational((1, r.index as i64).into()))
            } else {
                r.as_float().map(|x| x.ln().into())
            }
        }
        otherwise => natural_log(otherwise.as_float()?)
    }
}

fn recursive_e_count(x: &Number) -> Option<i64> {
    match x {
        Number::Symbol(Symbol::E) => Some(1),
        Number::Symbolic(s) => match &**s {
            Symbolic {coeff: Some(coeff), symbol, constant: None}
                if symbol == &Symbol::E => recursive_e_count(&coeff).map(|x| x + 1),
            Symbolic {coeff: None, symbol, constant: None} if symbol == &Symbol::E => Some(1),
            _ => None
        }
        _ => None
    }
}
