use std::{fmt, hash::Hash, ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign}};
use num_traits::{Bounded, One, Zero};
use rand::Rng;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize};
use crate::{InternalF, InternalI, models::time::Bound};

use super::TimeBound;

// Wrapper for f64 to implement extern traits
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, Hash)]
pub struct ClockValue(Option<InternalF>);

impl ClockValue {

    pub fn infinity() -> Self {
        ClockValue(Some(InternalF::MAX))
    }

    pub fn neg_infinity() -> Self {
        ClockValue(Some(InternalF::MIN))
    }

    pub fn disabled() -> Self {
        ClockValue(None)
    }

    pub fn is_infinite(&self) -> bool {
        match self.0 {
            Some(f) => f == InternalF::MAX || f == InternalF::MIN,
            None => false,
        }
    }

    pub fn is_disabled(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_enabled(&self) -> bool {
        !self.0.is_some()
    }

    pub fn disable(&mut self) {
        self.0 = None
    }

    #[inline]
    pub fn sample_distribution<R : Rng>(rng : &mut R, dist : &impl Distribution<f64>) -> Self {
        ClockValue::from(dist.sample(rng))
    }

    #[inline]
    pub fn distribution<R, D>(dist : D) -> Box<dyn Fn(&mut R) -> Self>
        where R : Rng, D : Distribution<f64> + 'static
    {
        Box::new(move |rng : &mut R| ClockValue::from(dist.sample(rng)))
    }

}

pub trait TimeType : Into<ClockValue> {}
impl<T : Into<ClockValue>> TimeType for T {}

impl Add for ClockValue {
    type Output = ClockValue;
    fn add(self, rhs: Self) -> Self::Output {
        if self.is_disabled() || rhs.is_disabled() {
            return Self::disabled();
        }
        if self.is_infinite() && 
            rhs.is_infinite() && 
            self.0.unwrap().signum() != rhs.0.unwrap().signum() 
        {
            panic!("Indeterminate sum !");
        }
        ClockValue(
            Some(self.0.unwrap().saturating_add(rhs.0.unwrap()))
        )
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
        ClockValue(self.0.map(|x| -x))
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
        if self.is_disabled() || rhs.is_disabled() {
            return Self::disabled();
        }
        ClockValue(
            Some(self.0.unwrap().saturating_mul(rhs.0.unwrap()))
        )
    }
}

impl MulAssign for ClockValue {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Mul<i64> for ClockValue {
    type Output = ClockValue;
    fn mul(self, rhs: i64) -> Self::Output {
        ClockValue(self.0.map(|c| c * rhs))
    }
}

impl MulAssign<i64> for ClockValue {
    fn mul_assign(&mut self, rhs: i64) {
        *self = *self * rhs
    }
}

impl Zero for ClockValue {
    fn zero() -> Self {
        ClockValue(Some(InternalF::ZERO))
    }
    fn is_zero(&self) -> bool {
        match self.0 {
            Some(f) => f.is_zero(),
            None => false,
        }
    }
}

impl One for ClockValue {
    fn one() -> Self {
        ClockValue(Some(InternalF::ONE))
    }
    fn is_one(&self) -> bool {
        match self.0 {
            Some(f) => f == InternalF::ONE,
            None => false,
        }
    }
}

impl fmt::Display for ClockValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(x) => x.fmt(f),
            None => write!(f, "_"),
        }
    }
}

impl Bounded for ClockValue {
    fn min_value() -> Self {
        ClockValue::neg_infinity()
    }
    fn max_value() -> Self {
        ClockValue::infinity()
    }
}

impl Default for ClockValue {
    fn default() -> Self {
        ClockValue::zero()
    }
}

impl From<f64> for ClockValue {
    fn from(value: f64) -> Self {
        if value.is_nan() {
            Self(None)
        } else if value == f64::INFINITY {
            Self::infinity()
        } else if value == f64::NEG_INFINITY {
            Self::neg_infinity()
        } else {
            Self(Some(InternalF::from_num(value)))
        }
    }
}

impl From<InternalI> for ClockValue {
    fn from(value: InternalI) -> Self {
        if value == InternalI::MAX {
            Self::infinity()
        } else if value == InternalI::MIN {
            Self::neg_infinity()
        } else {
            Self(Some(InternalF::from_num(value)))
        }
    }
}

impl<T : TimeType> From<Bound<T>> for ClockValue {
    fn from(value: Bound<T>) -> Self {
        match value {
            Bound::Infinite => ClockValue::infinity(),
            Bound::MinusInfinite => ClockValue::neg_infinity(),
            Bound::Large(x) => x.into(),
            Bound::Strict(x) => x.into()
        }
    }
}