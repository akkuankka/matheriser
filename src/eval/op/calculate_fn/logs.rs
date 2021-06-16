use crate::eval::{Data, Symbolic};

type DataResult = Result<Data, String>;

pub fn log_10(x: Data) -> DataResult {
    if x <= 0.into() {
        // x is positive
        return Err("Error: Logarithm of a non-positive number".to_string());
    }
    match x {
        Data::Int(n) => {
            if n % 10 == 0 {
                Ok(Data::from((n as f64).log10() as i64)) // cast is sane because this should be a whole number
            } else {
                Ok(Data::from((n as f64).log10()))
            }
        }
        Data::Float(n) => Ok(Data::Float(n.log10())),
        Data::Rational(r) => {
            // this is an optimisation for precision rather than speed, any cases where this would result in anything other than a float would get cancelled out anyway, but the more we can avoid floating-poing errors the better
            let (numer, denom) = (*r.numer(), *r.denom());
            log_10(Data::from(numer))? - log_10(Data::from(denom))?
        }
        otherwise => log_10(otherwise.as_float()?)
    }
}

pub fn natural_log(x: Data) -> DataResult {
    if x <= 0.into() {
        return Err("Error: Logarithm of a non-positive number".to_string())
    }
    match x {
        Data::Symbol(s) if s == "e" || s == "E" => Ok(Data::Int(1)),
        Data::Symbolic(s) => {
            let Symbolic {coeff, symbol, constant} = *s;
            if symbol == "e" || symbol == "E" {
                if let (Some(c), None) = (&coeff, &constant) {
                    if let Some(n) = recursive_e_count(&c) {
                        Ok(Data::Int(n + 1))
                    } else {
                        natural_log(c.clone()).and_then(|x| x + Data::from(1))
                    }
                } else if constant.is_none() && coeff.is_none() {
                    Ok(Data::Int(1))
                } else {
                    Symbolic {coeff, symbol, constant}.as_float().map(|x| Data::from(x.ln()))
                }
            } else {
                Symbolic {coeff, symbol, constant}.as_float().map(|x| Data::from(x.ln()))
            } 
        }
        Data::Radical(r) => {
            if r.coefficient == 1.into() && *r.radicand == Data::Symbol("e".into()) {
                Ok(Data::Rational((1, r.index as i64).into()))
            } else {
                r.as_float().map(|x| x.ln().into())
            }
        }
        otherwise => natural_log(otherwise.as_float()?)
    }
}

fn recursive_e_count(x: &Data) -> Option<i64> {
    match x {
        Data::Symbol(s) if s == "e" || s == "E" => Some(1),
        Data::Symbolic(s) => match &**s {
            Symbolic {coeff: Some(coeff), symbol, constant: None}
                if symbol == "e" || symbol == "E" => recursive_e_count(&coeff).map(|x| x + 1),
            Symbolic {coeff: None, symbol, constant: None} if symbol == "e" || symbol == "E" => Some(1),
            _ => None
        }
        _ => None
    }
}
