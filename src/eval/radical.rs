use super::{Data, DivisibleBy, ratio_as_float, op::root::NthRoot};
use num::{integer::Roots, rational::Ratio};
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Debug, PartialEq)]
pub struct Radical {
    pub coefficient: Ratio<i64>,
    pub index: u32,
    pub radicand: Box<Data>,
}

const PRIMES_TO_50: [u16; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];

fn primes_to_k<'a>(k: u16) -> Vec<u16>
{
    let primes = PRIMES_TO_50.iter().copied().take_while(move |n| *n < k).collect::<Vec<_>>(); 
   primes
}

impl Radical {
    pub fn simplify(self) -> Self {
        let prime_exponands = primes_to_k(self.radicand.nth_root(self.index as i64).into().ceil() as u16).iter().map(|x| (x.pow(self.index), *x as u16));
        let possible_extractible_factors: Vec<(_, _)> = prime_exponands
            .filter(|x| {
                let data = *self.radicand;
                data.divisible_by(x.1)
            })
            .collect();
        self.simplify_by(possible_extractible_factors)
    }

    fn simplify_by(self, factors: Vec<(u16, u16)>) -> Self {
        if factors.len() == 0 {
            return self;
        } //bad form, maybe
        let ([(factor, root)], factors) = factors.split_at(1);
        let coefficient = self.coefficient * *root as i64;
        let radicand = *self.radicand / (*factor as i64);
        Radical {
            coefficient,
            index: self.index,
            radicand,
        }
        .simplify_by(factors.to_vec())
    }
    fn new(coeff: Ratio<i64>, index: u32, radicand: Box<Data>) -> Self {
        Self {
            coefficient: coeff,
            index,
            radicand,
        }.simplify()
    }
}

impl DivisibleBy<Ratio<i64>> for Radical {
    fn divisible_by(self, rhs: Ratio<i64>) -> bool {
        self.coefficient.divisible_by(rhs)
    }
}

impl DivisibleBy<u16> for Radical {
    fn divisible_by(self, rhs: u16) -> bool {
        self.coefficient.numer().divisible_by(rhs as i64)
    }
}
impl DivisibleBy<i64> for Radical {
    fn divisible_by(self, rhs: i64) -> bool {
        self.coefficient.numer().divisible_by(rhs)
    }
}

impl DivisibleBy<Self> for Radical {
    fn divisible_by(self, rhs: Self) -> bool {
        // if we assume our radicals to be reduced, as I will, radicals are divisible if their radicands, indices are the same
        if *self.radicand == *rhs.radicand && self.index == rhs.index {
            self.coefficient.divisible_by(rhs.coefficient)
        } else {
            false
        }
    }
}

impl Radical {
    pub fn as_float(self) -> f64 {
        ratio_as_float(self.coefficient) * f64::from(*self.radicand).nth_root(self.index as f64)
    }
}
