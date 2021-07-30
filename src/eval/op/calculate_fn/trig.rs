use crate::eval::op::Div;
use crate::eval::{ratio_as_float, Number, Radical, SymbolEval, Symbolic, Symbol};
use num::rational::Ratio;
use std::convert::TryFrom;
use std::rc::Rc;

type DataResult = Result<Number, String>;

pub fn sin(data: Number) -> DataResult {
    fn sin_pi_coeff_lookup(c: &Number) -> Option<Number> {
        Some(match c {
            Number::Rational(r) => match (*r.numer(), *r.denom()) {
                (1, 6) => Number::Rational((1, 2).into()),
                (1, 4) => Number::Radical(Rc::new(Radical::new_raw((1, 2).into(), 2, &Number::from(2)))),
                (1, 3) => Number::Radical(Rc::new(Radical::new_raw((1, 2).into(), 2, &Number::from(3)))),
                (1, 2) => Number::Int(1),
                (1, 10) => Number::Symbolic(
                    Symbolic {
                        coeff: Number::Rational((1, 2).into()).into(),
                        symbol: Symbol::Phi,
                        constant: Number::Rational((-1, 2).into()).into(),
                    }
                    .into(),
                ),
                _ => return None,
            },
            Number::Int(_) => Number::Int(0),
            _ => return None,
        })
    }
    Ok(match data {
        Number::Int(0) => Number::Int(0),
        Number::Int(n) => (n as f64).sin().into(),
        Number::Float(n) => n.sin().into(),
        Number::Radical(n) => n.as_float()?.sin().into(),
        Number::Rational(n) => ratio_as_float(n).sin().into(),
        Number::Symbol(Symbol::Pi) => Number::Int(0),
        Number::Symbol(s) => s.symbol_eval()?.sin().into(),
        Number::Symbolic(a) => match &*a {
            Symbolic {
                coeff: Some(coeff),
                symbol,
                constant: None,
            } if symbol == &Symbol::Pi => {
                // if in terms of pi
                if coeff
                    <= &Number::Symbolic(
                        Symbolic {
                            coeff: Some(Number::Rational(Ratio::from((1, 2)))),
                            symbol: Symbol::Pi,
                            constant: None,
                        }
                        .into(),
                    )
                    && coeff >= &Number::Int(0)
                {
                    // if in quadrant 1
                    if let Some(ret) = sin_pi_coeff_lookup(&coeff) {
                        // is there a known and expressible identity for sin theta
                        ret
                    } else {
                        f64::try_from(Number::Symbolic(
                            // otherwise just do it as a float
                            Symbolic {
                                coeff: Some(coeff.clone()),
                                symbol: *symbol,
                                constant: None,
                            }
                            .into(),
                        ))?
                        .sin()
                        .into()
                    }
                } else {
                    // not in quadrant 1
                    match coeff {
                        Number::Int(_) => {
                            // is the coefficient going to have a known identity
                            Number::Int(0)
                        }
                        Number::Rational(r) => {
                            let in_unit = r % 2;
                            let not_negative = if in_unit < 0.into() {
                                Ratio::from(2) - in_unit
                            } else {
                                in_unit
                            };
                            // for the unit circle, if :theta = k*:pi, 0 < k < 2, if 0 < k < 1, sin :theta is positive
                            // the values in the 1st and 2nd quadrants are equal, 3rd and 4th are too, reflected about the y axis
                            if not_negative < 1.into() {
                                let reflected = if not_negative <= (1, 2).into() {
                                    not_negative
                                } else {
                                    Ratio::from(1) - not_negative
                                };
                                if let Some(n) = sin_pi_coeff_lookup(&Number::Rational(reflected)) {
                                    n
                                } else {
                                    f64::try_from(Number::Symbolic(
                                        // otherwise just do it as a float
                                        Symbolic {
                                            coeff: Some(coeff.clone()),
                                            symbol: *symbol,
                                            constant: None,
                                        }
                                        .into(),
                                    ))?
                                    .sin()
                                    .into()
                                }
                            } else {
                                // must be in quadrant 3 or 4
                                let rotated = not_negative - 1;
                                let reflected = if rotated <= (1, 2).into() {
                                    rotated
                                } else {
                                    Ratio::from(1) - rotated
                                };
                                if let Some(n) = sin_pi_coeff_lookup(&Number::Rational(reflected)) {
                                    -&n
                                } else {
                                    Number::from(
                                        -(f64::try_from(Number::Symbolic(
                                            // otherwise just do it as a float
                                            Symbolic {
                                                coeff: Some(coeff.clone()),
                                                symbol: *symbol,
                                                constant: None,
                                            }
                                            .into(),
                                        ))?
                                        .sin()),
                                    )
                                }
                            }
                        }
                        a => Number::from((f64::try_from(a.clone())? * std::f64::consts::PI).sin()),
                    }
                }
            }
            a => Number::from(a.clone().as_float()?.sin()),
        },
    })
}

pub fn cos(data: Number) -> DataResult {
    fn cos_pi_coeff_lookup(c: &Number) -> Option<Number> {
        Some(match c {
            Number::Rational(r) => match (*r.numer(), *r.denom()) {
                (1, 6) => Number::Radical(Rc::new(Radical::new_raw((1, 2).into(), 2, &Number::from(3)))),
                (1, 4) => Number::Radical(Rc::new(Radical::new_raw((1, 2).into(), 2, &Number::from(2).into()))),
                (1, 3) => Number::Rational((1, 2).into()),
                (1, 2) => Number::Int(0),
                (1, 5) => Number::Symbolic(
                    Symbolic {
                        coeff: Some(Number::Rational((1, 2).into())),
                        symbol: Symbol::Phi,
                        constant: None,
                    }
                    .into(),
                ),
                _ => return None,
            },
            Number::Int(_) => Number::Int(1), // this seems weird but remember that this is only being called in quadrant 1
            _ => return None,
        })
    }
    Ok(match data {
        Number::Int(0) => Number::Int(1),
        Number::Int(n) => (n as f64).cos().into(),
        Number::Float(n) => n.cos().into(),
        Number::Rational(n) => ratio_as_float(n).cos().into(),
        Number::Radical(n) => n.as_float()?.cos().into(),
        Number::Symbol(Symbol::Pi) => Number::Int(-1),
        Number::Symbol(s) => s.symbol_eval()?.cos().into(),
        Number::Symbolic(a) => match &*a {
            Symbolic {
                coeff: Some(coeff),
                symbol,
                constant: None,
            } if symbol == &Symbol::Pi => {
                // if in terms of pi
                if coeff
                    <= &Number::Symbolic(
                        Symbolic {
                            coeff: Some(Number::Rational(Ratio::from((1, 2)))),
                            symbol: Symbol::Pi,
                            constant: None,
                        }
                        .into(),
                    )
                    && coeff >= &Number::Int(0)
                {
                    // if in quadrant 1
                    if let Some(ret) = cos_pi_coeff_lookup(&coeff) {
                        // is there a known and expressible identity for cos theta
                        ret
                    } else {
                        f64::try_from(Number::Symbolic(
                            // otherwise just do it as a float
                            Symbolic {
                                coeff: Some(coeff.clone()),
                                symbol: *symbol,
                                constant: None,
                            }
                            .into(),
                        ))?
                        .cos()
                        .into()
                    }
                } else {
                    // not in quadrant 1
                    match coeff {
                        // is the coefficient going to have a known identity
                        Number::Int(a) => {
                            if a % 2 == 0 {
                                Number::Int(1)
                            } else {
                                Number::Int(-1)
                            }
                        }
                        Number::Rational(r) => {
                            let in_unit = r % 2;
                            let not_negative = if in_unit < 0.into() {
                                Ratio::from(2) - in_unit
                            } else {
                                in_unit
                            };
                            if not_negative <= (1, 2).into() || not_negative >= (3, 2).into() {
                                // quadrants 1 and 4
                                let reflected = if not_negative <= (1, 2).into() {
                                    not_negative
                                } else {
                                    Ratio::from(2) - not_negative // reflect quadrant 4 about the x axis
                                };
                                if let Some(n) = cos_pi_coeff_lookup(&Number::Rational(reflected)) {
                                    n
                                } else {
                                    f64::try_from(Number::Symbolic(
                                        // otherwise just do it as a float
                                        Symbolic {
                                            coeff: Some(coeff.clone()),
                                            symbol: *symbol,
                                            constant: None,
                                        }
                                        .into(),
                                    ))?
                                    .cos()
                                    .into()
                                }
                            } else {
                                // must be in quadrant 2 or 3
                                let rotated = if not_negative > 1.into() {
                                    // quadrant 2
                                    Ratio::from(1) - not_negative
                                } else if not_negative > (3, 2).into() {
                                    // quadrant 3
                                    not_negative - Ratio::from(1)
                                } else {
                                    unreachable!()
                                };
                                if let Some(n) = cos_pi_coeff_lookup(&Number::Rational(rotated)) {
                                    -&n
                                } else {
                                    Number::from(
                                        -(f64::try_from(Number::Symbolic(
                                            // otherwise just do it as a float
                                            Symbolic {
                                                coeff: Some(coeff.clone()),
                                                symbol: *symbol,
                                                constant: None,
                                            }
                                            .into(),
                                        ))?
                                        .cos()),
                                    )
                                }
                            }
                        }
                        a => Number::from((f64::try_from(a.clone())? * std::f64::consts::PI).cos()),
                    }
                }
            }
            a => Number::from(a.clone().as_float()?.cos()),
        },
    })
}

pub fn tan(theta: Number) -> DataResult {
    fn tan_pi_coeff_lookup(coeff: Ratio<i64>) -> Option<Number> {
        Some(match (*coeff.numer(), *coeff.denom()) {
            (1, 6) => Number::Radical(Rc::new(Radical::new_raw((1, 3).into(), 2, &Number::from(3)))),
            (1, 4) => Number::Int(1),
            (1, 3) => Number::Radical(Rc::new(Radical::new_raw(1.into(), 2, &Number::from(3)))),
            (1, 2) => return None,
            _ => return None,
        })
    }

    match theta {
        Number::Int(0) => Ok(Number::Int(0)),
        Number::Symbolic(a) => match &*a {
            Symbolic {
                coeff: Some(coeff),
                symbol,
                constant: None,
            } if symbol == &Symbol::Pi => match coeff {
                Number::Int(_) => Ok(Number::Int(0)),
                Number::Rational(r) => {
                    let r = r % 1;
                    if r == (1, 2).into() || r == (-1, 2).into() {
                        return Err("Undefined: Tangent of 1/2".to_string())
                    }
                    let looked_up = if r < 0.into() {
                        tan_pi_coeff_lookup(r).map(|x| -&x)
                    } else {
                        tan_pi_coeff_lookup(r)
                    };                      
                    if let Some(res) = looked_up {
                        Ok(res)
                    }
                    else {sin(Number::Rational(r.clone()))?.div(&cos(Number::Rational(r))?)}
                },
                otherwise => Ok(f64::try_from(otherwise.clone())?.tan().into())
            },
            otherwise => {
                Ok(otherwise.as_float()?.tan().into())
            }
        },
        otherwise => sin(otherwise.clone())?.div(&cos(otherwise)?)
    }
}
