use std::{fmt, hash::Hash, ops::{Add, AddAssign, Mul, MulAssign, Neg, Range, Sub, SubAssign}};
use num_traits::{One, Zero};
use rand::{distributions::{uniform::{SampleBorrow, SampleRange, SampleUniform, UniformFloat, UniformSampler}, Distribution, Standard}, Rng, RngCore};
use serde::{Deserialize, Serialize};
use super::TimeBound;

use TimeBound::*;

// Wrapper for f64 to implement extern traits
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ClockValue(pub f64);

impl ClockValue {

    pub fn infinity() -> Self {
        ClockValue(f64::INFINITY)
    }

    pub fn disabled() -> Self {
        ClockValue(f64::NAN)
    }

    pub fn is_infinite(&self) -> bool {
        self.0.is_infinite()
    }

    pub fn is_disabled(&self) -> bool {
        self.0.is_nan()
    }

    pub fn is_enabled(&self) -> bool {
        !self.0.is_nan()
    }

}

impl Add for ClockValue {
    type Output = ClockValue;
    fn add(self, rhs: Self) -> Self::Output {
        ClockValue(self.0 + rhs.0)
    }
}

impl AddAssign for ClockValue {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Neg for ClockValue {
    type Output = ClockValue;
    fn neg(self) -> Self::Output {
        ClockValue(-self.0)
    }
}

impl Sub for ClockValue {
    type Output = ClockValue;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl SubAssign for ClockValue {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul for ClockValue {
    type Output = ClockValue;
    fn mul(self, rhs: Self) -> Self::Output {
        ClockValue(self.0 * rhs.0)
    }
}

impl MulAssign for ClockValue {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Zero for ClockValue {
    fn zero() -> Self {
        ClockValue(f64::zero())
    }
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl One for ClockValue {
    fn one() -> Self {
        ClockValue(f64::one())
    }
    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

impl fmt::Display for ClockValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Hash for ClockValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ((self.0 * 100_000_000.0) as u64).hash(state)
    }
}

impl Distribution<ClockValue> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ClockValue {
        let rand_f = rng.gen();
        ClockValue(rand_f)
    }
}

/*impl SampleRange<ClockValue> for Range<ClockValue> {
    fn sample_single<R: RngCore + ?Sized>(self, rng: &mut R) -> ClockValue {
        let f_range = (self.start.0)..(self.end.0);
        let rand_f = rng.gen_range(f_range);
        ClockValue(rand_f)
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}*/

pub struct UniformClockValue(UniformFloat<f64>);

impl UniformSampler for UniformClockValue {
    type X = ClockValue;
    fn new<B1, B2>(low: B1, high: B2) -> Self
        where B1: SampleBorrow<Self::X> + Sized,
              B2: SampleBorrow<Self::X> + Sized
    {
        UniformClockValue(UniformFloat::<f64>::new(low.borrow().0, high.borrow().0))
    }
    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
        where B1: SampleBorrow<Self::X> + Sized,
              B2: SampleBorrow<Self::X> + Sized
    {
        UniformClockValue(UniformFloat::<f64>::new_inclusive(
            low.borrow().0,
            high.borrow().0,
        ))
    }
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        ClockValue(self.0.sample(rng))
    }
}

impl SampleUniform for ClockValue {
    type Sampler = UniformClockValue;
}

impl From<TimeBound> for ClockValue {
    fn from(value: TimeBound) -> Self {
        match value {
            Infinite => ClockValue::infinity(),
            MinusInfinite => ClockValue(f64::NEG_INFINITY),
            Large(x) => ClockValue(x as f64),
            Strict(x) => ClockValue(x as f64)
        }
    }
}