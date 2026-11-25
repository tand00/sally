use std::{fmt, hash::Hash, ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign}};
use num_traits::{One, Zero};
use rand::Rng;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize};
use crate::real::Real;


// Wrapper for f64 to implement extern traits
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize, Hash)]
pub enum ClockValue {
    Disabled,
    Enabled(Real)
}

impl ClockValue {

    pub fn is_disabled(&self) -> bool {
        matches!(self, ClockValue::Disabled)
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self, ClockValue::Enabled(_))
    }

    pub fn disable(&mut self) {
        *self = ClockValue::Disabled
    }

    pub fn map<F>(self, f: F) -> Self
        where F: Fn(Real) -> Real
    {
        match self {
            ClockValue::Disabled => ClockValue::Disabled,
            ClockValue::Enabled(x) => ClockValue::Enabled(f(x))
        }
    }

}

impl Add for ClockValue {
    type Output = ClockValue;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ClockValue::Enabled(x), ClockValue::Enabled(y)) => 
                ClockValue::Enabled(x + y),
            _ => ClockValue::Disabled
        }
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
        self.map(|x| -x)
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
        match (self, rhs) {
            (ClockValue::Enabled(x), ClockValue::Enabled(y)) => 
                ClockValue::Enabled(x * y),
            _ => ClockValue::Disabled
        }
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
        self.map(|c| c * rhs.into())
    }
}

impl MulAssign<i64> for ClockValue {
    fn mul_assign(&mut self, rhs: i64) {
        *self = *self * rhs
    }
}

impl Zero for ClockValue {
    fn zero() -> Self {
        ClockValue::Enabled(Real::zero())
    }
    fn is_zero(&self) -> bool {
        match self {
            ClockValue::Enabled(f) => f.is_zero(),
            ClockValue::Disabled => false,
        }
    }
}

impl One for ClockValue {
    fn one() -> Self {
        ClockValue::Enabled(Real::one())
    }
}

impl fmt::Display for ClockValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClockValue::Enabled(x) => x.fmt(f),
            ClockValue::Disabled => write!(f, "_"),
        }
    }
}

impl Default for ClockValue {
    fn default() -> Self {
        ClockValue::zero()
    }
}

impl From<Real> for ClockValue {
    fn from(value: Real) -> Self {
        ClockValue::Enabled(value)
    }
}