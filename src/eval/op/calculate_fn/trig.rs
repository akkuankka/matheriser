use crate::eval::{ratio_as_float, Data, Radical, SymbolEval, Symbolic};
use num::rational::Ratio;
use std::convert::TryFrom;

type DataResult = Result<Data, String>;

pub fn sin(data: Data) -> DataResult {
    fn sin_pi_coeff_lookup(c: &Data) -> Option<Data> {
        Some(match c {
            Data::Rational(r) => match (*r.numer(), *r.denom()) {
                (1, 6) => Data::Rational((1, 2).into()),
                (1, 4) => Data::Radical(Radical::new_raw((1, 2).into(), 2, Data::from(2).into())),
                (1, 3) => Data::Radical(Radical::new_raw((1, 2).into(), 2, Data::from(3).into())),
                (1, 2) => Data::Int(1),
                (1, 10) => Data::Symbolic(
                    Symbolic {
                        coeff: Data::Rational((1, 2).into()).into(),
                        symbol: "phi".into(),
                        constant: Data::Rational((-1, 2).into()).into(),
                    }
                    .into(),
                ),
                _ => return None,
            },
            Data::Int(_) => Data::Int(0),
            _ => return None,
        })
    }
    Ok(match data {
        Data::Int(0) => Data::Int(0),
        Data::Int(n) => (n as f64).sin().into(),
        Data::Float(n) => n.sin().into(),
        Data::Radical(n) => n.as_float()?.sin().into(),
        Data::Rational(n) => ratio_as_float(n).sin().into(),
        Data::Symbol(pi) if pi == "pi" => Data::Int(0),
        Data::Symbol(s) => s.symbol_eval()?.sin().into(),
        Data::Symbolic(a) => match *a {
            Symbolic {
                coeff: Some(coeff),
                symbol,
                constant: None,
            } if symbol == "pi" => {
                // if in terms of pi
                if coeff
                    <= Data::Symbolic(
                        Symbolic {
                            coeff: Some(Data::Rational(Ratio::from((1, 2)))),
                            symbol: "pi".into(),
                            constant: None,
                        }
                        .into(),
                    )
                    && coeff >= Data::Int(0)
                {
                    // if in quadrant 1
                    if let Some(ret) = sin_pi_coeff_lookup(&coeff) {
                        // is there a known and expressible identity for sin theta
                        ret
                    } else {
                        f64::try_from(Data::Symbolic(
                            // otherwise just do it as a float
                            Symbolic {
                                coeff: Some(coeff),
                                symbol,
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
                        Data::Int(_) => {
                            // is the coefficient going to have a known identity
                            Data::Int(0)
                        }
                        Data::Rational(r) => {
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
                                if let Some(n) = sin_pi_coeff_lookup(&Data::Rational(reflected)) {
                                    n
                                } else {
                                    f64::try_from(Data::Symbolic(
                                        // otherwise just do it as a float
                                        Symbolic {
                                            coeff: Some(coeff),
                                            symbol,
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
                                if let Some(n) = sin_pi_coeff_lookup(&Data::Rational(reflected)) {
                                    -n
                                } else {
                                    Data::from(
                                        -(f64::try_from(Data::Symbolic(
                                            // otherwise just do it as a float
                                            Symbolic {
                                                coeff: Some(coeff),
                                                symbol,
                                                constant: None,
                                            }
                                            .into(),
                                        ))?
                                        .sin()),
                                    )
                                }
                            }
                        }
                        a => Data::from((f64::try_from(a)? * std::f64::consts::PI).sin()),
                    }
                }
            }
            a => Data::from(a.as_float()?.sin()),
        },
    })
}

pub fn cos(data: Data) -> DataResult {
    fn cos_pi_coeff_lookup(c: &Data) -> Option<Data> {
        Some(match c {
            Data::Rational(r) => match (*r.numer(), *r.denom()) {
                (1, 6) => Data::Radical(Radical::new_raw((1, 2).into(), 2, Data::from(3).into())),
                (1, 4) => Data::Radical(Radical::new_raw((1, 2).into(), 2, Data::from(2).into())),
                (1, 3) => Data::Rational((1, 2).into()),
                (1, 2) => Data::Int(0),
                (1, 5) => Data::Symbolic(
                    Symbolic {
                        coeff: Some(Data::Rational((1, 2).into())),
                        symbol: "phi".into(),
                        constant: None,
                    }
                    .into(),
                ),
                _ => return None,
            },
            Data::Int(_) => Data::Int(1), // this seems weird but remember that this is only being called in quadrant 1
            _ => return None,
        })
    }
    Ok(match data {
        Data::Int(0) => Data::Int(1),
        Data::Int(n) => (n as f64).cos().into(),
        Data::Float(n) => n.sin().into(),
        Data::Rational(n) => ratio_as_float(n).sin().into(),
        Data::Radical(n) => n.as_float()?.cos().into(),
        Data::Symbol(pi) if pi == "pi" => Data::Int(0),
        Data::Symbol(s) => s.symbol_eval()?.cos().into(),
        Data::Symbolic(a) => match *a {
            Symbolic {
                coeff: Some(coeff),
                symbol,
                constant: None,
            } if symbol == "pi" => {
                // if in terms of pi
                if coeff
                    <= Data::Symbolic(
                        Symbolic {
                            coeff: Some(Data::Rational(Ratio::from((1, 2)))),
                            symbol: "pi".into(),
                            constant: None,
                        }
                        .into(),
                    )
                    && coeff >= Data::Int(0)
                {
                    // if in quadrant 1
                    if let Some(ret) = cos_pi_coeff_lookup(&coeff) {
                        // is there a known and expressible identity for cos theta
                        ret
                    } else {
                        f64::try_from(Data::Symbolic(
                            // otherwise just do it as a float
                            Symbolic {
                                coeff: Some(coeff),
                                symbol,
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
                        // is the coefficient going to have a known identity
                        Data::Int(a) => {
                            if a % 2 == 0 {
                                Data::Int(1)
                            } else {
                                Data::Int(-1)
                            }
                        }
                        Data::Rational(r) => {
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
                                if let Some(n) = cos_pi_coeff_lookup(&Data::Rational(reflected)) {
                                    n
                                } else {
                                    f64::try_from(Data::Symbolic(
                                        // otherwise just do it as a float
                                        Symbolic {
                                            coeff: Some(coeff),
                                            symbol,
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
                                if let Some(n) = cos_pi_coeff_lookup(&Data::Rational(rotated)) {
                                    -n
                                } else {
                                    Data::from(
                                        -(f64::try_from(Data::Symbolic(
                                            // otherwise just do it as a float
                                            Symbolic {
                                                coeff: Some(coeff),
                                                symbol,
                                                constant: None,
                                            }
                                            .into(),
                                        ))?
                                        .cos()),
                                    )
                                }
                            }
                        }
                        a => Data::from((f64::try_from(a)? * std::f64::consts::PI).cos()),
                    }
                }
            }
            a => Data::from(a.as_float()?.cos()),
        },
    })
}

pub fn tan(theta: Data) -> DataResult {
    fn tan_pi_coeff_lookup(coeff: Ratio<i64>) -> Option<Data> {
        Some(match (*coeff.numer(), *coeff.denom()) {
            (1, 6) => Data::Radical(Radical::new_raw((1, 3).into(), 2, Data::from(3).into())),
            (1, 4) => Data::Int(1),
            (1, 3) => Data::Radical(Radical::new_raw(1.into(), 2, Data::from(3).into())),
            (1, 2) => return None,
            _ => return None,
        })
    }

    match theta {
        Data::Int(0) => Ok(Data::Int(0)),
        Data::Symbolic(a) => match *a {
            Symbolic {
                coeff: Some(coeff),
                symbol,
                constant: None,
            } if symbol == "pi" => match coeff {
                Data::Int(_) => Ok(Data::Int(0)),
                Data::Rational(r) => {
                    let r = r % 1;
                    if r == (1, 2).into() || r == (-1, 2).into() {
                        return Err("Undefined: Tangent of 1/2".to_string())
                    }
                    let looked_up = if r < 0.into() {
                        tan_pi_coeff_lookup(r).map(|x| -x)
                    } else {
                        tan_pi_coeff_lookup(r)
                    };                      
                    if let Some(res) = looked_up {
                        Ok(res)
                    }
                    else {sin(Data::Rational(r.clone()))? / cos(Data::Rational(r))?}
                },
                otherwise => Ok(f64::try_from(otherwise)?.tan().into())
            },
            otherwise => {
                Ok(otherwise.as_float()?.tan().into())
            }
        },
        otherwise => sin(otherwise.clone())? / cos(otherwise)?
    }
}
