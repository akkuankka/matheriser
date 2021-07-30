use crate::eval::{
    op::{Div, Mul, Pow, Sub},
    ratio_as_float, Number, SymbolEval, Symbolic,
};
use num::integer::lcm;
use num::rational::Ratio;
use std::cmp::Ordering;

impl std::cmp::PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (&self, &other) {
            (Self::Int(a), Self::Int(b)) => Some(a.cmp(b)),
            (Self::Float(a), Self::Int(b)) => a.partial_cmp(&(*b as f64)),
            (Self::Rational(a), Self::Int(b)) => a.partial_cmp(&Ratio::from(*b)),
            (Self::Symbol(a), Self::Int(b)) => a.symbol_eval().ok()?.partial_cmp(&(*b as f64)),
            (Self::Radical(a), Self::Int(b)) => {
                let (index, radicand) = (a.index, a.radicand.clone());
                // println!("{} root {:?} ? {}", index, radicand, b); i don't know what this is used for?
                let (lneg, rneg) = (a.coefficient < 0.into(), *b < 0);
                let rhs = Ratio::from(*b) / a.coefficient;
                let result = if lneg { -&radicand } else { radicand }.partial_cmp(&if rneg {
                    -&Self::Rational(rhs.pow(index as i32).abs())
                } else {
                    Self::Rational(rhs.pow(index as i32).abs())
                });
                println!("{:?}", result);
                result
            }
            (Self::Symbolic(a), Self::Int(b)) => {
                let Symbolic {
                    ref coeff,
                    symbol,
                    ref constant,
                } = **a;
                let should_flip = coeff.as_ref().unwrap_or(&Self::Int(1)) < &Number::from(0);
                Self::Symbol(symbol)
                    .partial_cmp(
                        &(Self::Int(*b)
                            .sub(&constant.clone().unwrap_or(Number::Int(0)))
                            .ok()?
                            .div(&coeff.clone().unwrap_or(Number::Int(1))))
                        .ok()?,
                    )
                    .map(|o| if should_flip { o.reverse() } else { o })
            }
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b),
            (Self::Symbol(a), Self::Symbol(b)) => {
                a.symbol_eval().ok()?.partial_cmp(&b.symbol_eval().ok()?)
            }
            (Self::Symbol(a), Self::Float(b)) => a.symbol_eval().ok()?.partial_cmp(b),
            (Self::Float(a), &Self::Rational(b)) => a.partial_cmp(&ratio_as_float(*b)),
            (Self::Rational(a), Self::Symbol(b)) => {
                ratio_as_float(*a).partial_cmp(&b.symbol_eval().ok()?)
            }
            (Self::Symbol(a), Self::Radical(b)) => a
                .symbol_eval()
                .ok()?
                .partial_cmp(&b.clone().as_float().ok()?),
            (Self::Symbol(a), Self::Symbolic(b)) => {
                if (a == &b.symbol) && b.constant == None {
                    // symbol can be factored out
                    Number::Int(1).partial_cmp(&(b.coeff.clone().unwrap_or(Number::Int(1))))
                } else {
                    a.symbol_eval()
                        .ok()?
                        .partial_cmp(&b.clone().as_float().ok()?)
                }
            }
            (Self::Symbolic(a), Self::Symbolic(b)) => {
                // let a = a.clone();
                // let b = b.clone();
                if a.symbol == b.symbol {
                    // the symbols are the same: fast path
                    match (&a.constant, &b.constant) {
                        // keeping in mind that we're assuming our symbols are positive (this means symbols can't be used for variables)
                        (None, None) => a
                            .coeff.clone()
                            .unwrap_or(Number::Int(1))
                            .partial_cmp(&b.coeff.clone().unwrap_or(Number::Int(1))),
                        (Some(c), None) => {
                            let m = a.coeff.clone().unwrap_or(Number::Int(1));
                            let n = b.coeff.clone().unwrap_or(Number::Int(1));

                            if (m > n) && (c >= &0.into()) {
                                Some(Ordering::Greater)
                            } else if (m < n) && (c <= &0.into()) {
                                Some(Ordering::Less)
                            } else if (m == n) && (c == &0.into()) {
                                Some(Ordering::Equal)
                            } else if m > n {
                                if c > &m.sub(&n).ok()?.mul(&a.symbol.into()).ok()? {
                                    Some(Ordering::Greater)
                                } else {
                                    Some(Ordering::Less)
                                }
                            } else if m < n {
                                if c < &n.sub(&m).ok()?.mul(&a.symbol.into()).ok()? {
                                    Some(Ordering::Less)
                                } else {
                                    Some(Ordering::Greater)
                                }
                            } else {
                                unreachable!()
                            }
                        }
                        (None, Some(_)) => other.partial_cmp(&self).map(|o| o.reverse()),
                        (Some(c), Some(d)) => {
                            // let c = c.clone();
                            // let d = d.clone();
                            let m = a.coeff.clone().unwrap_or(Number::Int(1));
                            let n = b.coeff.clone().unwrap_or(Number::Int(1));
                            let x = Number::from(a.symbol);
                            if (m == n) && (c == d) {
                                Some(Ordering::Equal)
                            } else if x.mul(&m.sub(&n).ok()?).ok()? == d.sub(c).ok()? {
                                Some(Ordering::Equal)
                            } else if x.mul(&m.sub(&n).ok()?).ok()? < d.sub(c).ok()? {
                                Some(Ordering::Less)
                            } else if x.mul(&m.sub(&n).ok()?).ok()? < d.sub(&c).ok()? {
                                Some(Ordering::Less)
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    a.as_float().partial_cmp(&b.as_float())
                }
            }
            (Self::Float(a), Self::Radical(b)) => a.partial_cmp(&b.as_float().ok()?),
            (Self::Radical(a), Self::Radical(b)) => {
                // let a = a.clone();
                // let b = b.clone();
                // println!("{:?} is being compared with {:?}", a, b);
                if a.index == b.index {
                    // easily done, this will be nearly every case because this is mostly sqrts
                    let i = a.index;
                    let (lneg, rneg) = (a.coefficient < 0.into(), b.coefficient < 0.into());
                    let mut should_flip = a.radicand < Self::Int(0);
                    should_flip ^= b.coefficient < 0.into(); // the rarely used XOR-Assignment operator, both is true, it should be false
                    let m = a.coefficient;
                    let n = b.coefficient;
                    let mpow = m.pow(i as i32);
                    let npow = n.pow(i as i32);
                    // println!("SameIndex_Coeffs: {:?} / {:?} = ...", mpow, npow);
                    let lhs = Self::Rational(mpow / npow);
                    // println!("{:?}", lhs);
                    // println!(
                    //     "SameIndex_Radicands: {:?} / {:?} = ...",
                    //     b.radicand, a.radicand
                    // );
                    let r_rhs = b.radicand.div(&a.radicand).ok()?;
                    // println!("{:?}", r_rhs);
                    let result = lhs.partial_cmp(&r_rhs).map(|o| {
                        if should_flip {
                            o.reverse()
                        } else if lneg != rneg && o == Ordering::Equal {
                            if lneg {
                                Ordering::Less
                            } else {
                                Ordering::Greater
                            }
                        } else {
                            o
                        }
                    }); // if should flip, flip it
                        // println!("{:?}", result);
                    result
                } else {
                    let (lneg, rneg) = (a.coefficient < 0.into(), b.coefficient < 0.into());
                    let k = lcm(a.index, b.index) as i32; // lowest common multiple of the indices
                    let m = a.coefficient.abs();
                    let n = b.coefficient.abs();
                    let lhs = { |x: &Number| if lneg { -x } else { x.clone() } }(
                        // flip if negative
                        &Self::Rational(m.pow(k))
                            .mul(
                                &a.radicand
                                    .pow(&Number::from(k as i64 / a.index as i64))
                                    .ok()?,
                            )
                            .ok()?,
                    );
                    let rhs = { |x: &Number| if rneg { -x } else { x.clone() } }(
                        &Self::Rational(n.pow(k))
                            .mul(
                                &b.radicand
                                    .pow(&Number::from(k as i64 / b.index as i64))
                                    .ok()?,
                            )
                            .ok()?,
                    );
                    lhs.partial_cmp(&rhs)
                }
            }
            (Self::Radical(a), Self::Rational(b)) => {
                let (index, radicand) = (a.index, a.radicand.clone());
                let (lneg, rneg) = (a.coefficient < 0.into(), b < &0.into());
                let rhs = b / a.coefficient.abs();
                if lneg { -&radicand } else { radicand }.partial_cmp(&if rneg {
                    -&Self::Rational(rhs.pow(index as i32).abs())
                } else {
                    Self::Rational(rhs.pow(index as i32).abs())
                })
            }
            (Self::Rational(a), Self::Rational(b)) => a.partial_cmp(&b),
            (Self::Symbolic(a), Self::Rational(b)) => {
                let Symbolic {
                    coeff,
                    symbol,
                    constant,
                } = &**a;
                let should_flip = coeff.as_ref().unwrap_or(&Self::Int(1)) < &Number::from(0);
                Self::Symbol(*symbol)
                    .partial_cmp(
                        &(Self::Rational(*b)
                            .sub(&constant.clone().unwrap_or(Number::Int(0)))
                            .ok()?
                            .div(&coeff.clone().unwrap_or(Number::Int(1))))
                        .ok()?,
                    )
                    .map(|o| if should_flip { o.reverse() } else { o })
            }
            (&Self::Radical(a), &Self::Symbolic(b)) => {
                a.clone()
                    .as_float()
                    .ok()?
                    .partial_cmp(&b.clone().as_float().ok()?) // both of these values are sort of diffuse, but they're diffuse in different ways, so it's best to just use float
            }
            (&Self::Symbolic(a), &Self::Float(b)) => {
                a.clone().as_float().ok()?.partial_cmp(b) // both of these values are sort of diffuse, but they're diffuse in different ways, so it's best to just use float
            }
            (a, b) => b.partial_cmp(a).map(|o| o.reverse()),
        }
    }
}

trait Abs {
    fn abs(self) -> Self;
}

impl Abs for Ratio<i64> {
    fn abs(self) -> Self {
        let (n, d) = self.into();
        (n.abs(), d.abs()).into()
    }
}

#[cfg(test)]
mod test {
    use crate::eval::{radical::Radical, Number, Symbol, Symbolic};
    use num::rational::Ratio;
    use rand::Rng;
    use std::rc::Rc;
    #[test]
    fn ints() {
        let mut rng = rand::thread_rng();

        for i in 1..30 {
            let k: i64 = rng.gen_range(-256..255);
            if k == 0 {
                continue;
            }
            assert_eq!(
                i.partial_cmp(&(i * k)),
                Number::from(i).partial_cmp(&Number::from(i * k))
            )
        }
    }
    #[test]
    fn roots() {
        assert!(
            Number::Radical(Rc::new(Radical::new(3.into(), 2, &Number::from(2)))).partial_cmp(
                &Number::Radical(Rc::new(Radical::new(4.into(), 2, &Number::from(2))))
            ) == Some(std::cmp::Ordering::Less)
        ); // distinguish by coefficient
        assert!(
            Number::Int(2) > Number::Radical(Rc::new(Radical::new(1.into(), 2, &Number::from(2))))
        ); // distinguish from int
        assert!(
            Number::Radical(Rc::new(Radical::new(1.into(), 2, &Number::from(19))))
                > Number::Radical(Rc::new(Radical::new(3.into(), 2, &Number::from(2))))
        ); // distinguish complex
    }

    #[test]
    fn roots_negatives_cursory() {
        assert!(
            Number::Radical(Rc::new(Radical::new_raw(
                Ratio::from(-1),
                2,
                &Number::from(2)
            ))) < Number::Radical(Rc::new(Radical::new_raw(
                Ratio::from(1),
                2,
                &Number::from(2)
            )))
        );
        assert!(
            Number::Radical(Rc::new(Radical::new_raw(
                Ratio::from(-90),
                3,
                &Number::from(-2)
            ))) > Number::Radical(Rc::new(Radical::new_raw(
                Ratio::from(-1),
                3,
                &Number::from(2)
            )))
        );
    }

    #[test]
    fn roots_negatives_thorough() {
        // negatives work correctly
        let signs: [(i64, i64, i64, i64); 9] = [
            (1, 1, 1, 1),
            (-1, -1, 1, 1),
            (1, -1, -1, 1),
            (1, -1, 1, -1),
            (-1, 1, 1, -1),
            (-1, -1, 1, 1),
            (-1, -1, -1, -1),
            (1, -1, -1, -1),
            (-1, 1, 1, 1),
        ];
        let signs_with_ordering: Vec<_> = signs
            .iter()
            .map(|&(m, a, n, b)| ((m, a, n, b), (2 * m * a).partial_cmp(&(n * b))))
            .collect();
        println!("got signs");
        for ((m, a, n, b), ordering) in signs_with_ordering {
            println!("round");
            assert_eq!(
                {
                    let lhs = Number::Radical(Rc::new(Radical::new(
                        (2 * m).into(),
                        3,
                        &Number::from(2 * a),
                    )));
                    // println!("lhs: {:?}", lhs);
                    let rhs =
                        Number::Radical(Rc::new(Radical::new(n.into(), 3, &Number::from(2 * b))));
                    // println!("rhs: {:?}", rhs);
                    lhs.partial_cmp(&rhs)
                },
                ordering
            )
        }
    }

    #[test]
    fn symbolics() {
        assert!(
            Number::Symbolic(
                Symbolic {
                    coeff: Some(Number::from(2)),
                    symbol: Symbol::Pi,
                    constant: None
                }
                .into()
            ) > Number::Symbol(Symbol::Pi)
        );

        assert!(
            Number::Symbolic(
                Symbolic {
                    coeff: Some(Number::from(4)),
                    symbol: Symbol::Pi,
                    constant: None
                }
                .into()
            ) > Number::Symbolic(
                Symbolic {
                    coeff: Some(Number::from(2)),
                    symbol: Symbol::Pi,
                    constant: None
                }
                .into()
            )
        );
        assert!(
            Number::Symbolic(
                Symbolic {
                    coeff: Some(Number::from(2)),
                    symbol: Symbol::Pi,
                    constant: None
                }
                .into()
            ) > Number::from(6)
        );
    }
}
