use num_traits::Zero;
use rand::{distributions::{Distribution, WeightedIndex}, Rng};
use serde::{Deserialize, Serialize};

use rand_distr::{Exp, Gamma, Geometric, Normal, Uniform};

use crate::models::time::ClockValue;

#[derive(Debug, Clone)]
pub struct ProbabilisticChoice<T>(pub Vec<(T, f64)>, WeightedIndex<f64>);

impl<T> ProbabilisticChoice<T> {

    pub fn new(values : Vec<(T, f64)>) -> Self {
        let dist = WeightedIndex::new(values.iter().map(|x| x.1)).unwrap();
        ProbabilisticChoice(values, dist)
    }

    pub fn normalized(&self) -> Self
        where T : Clone
    {
        let sum : f64 = self.0.iter().map(|x| x.1).sum();
        Self::new(self.0.iter().map(|x| {
            (x.0.clone(), x.1 / sum)
        }).collect())
    }

    pub fn sample(&self, rng : &mut impl Rng) -> &T {
        let dist = &self.1;
        let sample = dist.sample(rng);
        &self.0[sample].0
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RealDistribution {
    Dirac(f64),
    Uniform(f64, f64),
    Exp(f64),
    Gamma(f64, f64),
    Erlang(i32, f64),
    Normal(f64, f64),
    UniformInt(i32, i32),
    Geometric(f64)
}

impl RealDistribution {

    pub fn urgent() -> Self {
        Self::Dirac(0.0)
    }

    pub fn sample(&self, rng : &mut impl Rng) -> f64 {
        match self {
            Self::Dirac(x) => *x,
            Self::Uniform(a, b) => Uniform::new(*a,* b).sample(rng),
            Self::Exp(r) => Exp::new(*r).unwrap().sample(rng),
            Self::Gamma(shape, scale) => Gamma::new(*shape, *scale).unwrap().sample(rng),
            Self::Erlang(shape, scale) => Gamma::new(*shape as f64, *scale).unwrap().sample(rng),
            Self::Normal(mean, std_dev) => Normal::new(*mean, *std_dev).unwrap().sample(rng),
            Self::UniformInt(a, b) => Uniform::new(*a,* b).sample(rng) as f64, 
            Self::Geometric(r) => Geometric::new(*r).unwrap().sample(rng) as f64
        }
    }

    pub fn sample_date(&self, rng : &mut impl Rng) -> ClockValue {
        let date = self.sample(rng);
        if date < 0.0 {
            ClockValue::zero()
        } else {
            ClockValue::from(date)
        }
    }

}

impl Default for RealDistribution {
    fn default() -> Self {
        Self::Dirac(1.0)
    }
}