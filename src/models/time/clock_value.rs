use std::{fmt, hash::Hash, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign}};
use num_traits::{One, Zero};
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

    pub fn map_mut<F>(&mut self, f: F)
        where F: Fn(&mut Real)
    {
        match self {
            ClockValue::Disabled => (),
            ClockValue::Enabled(x) => f(x)
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

impl<T: Into<Real>> Add<T> for ClockValue {
    type Output = ClockValue;
    fn add(self, rhs: T) -> Self::Output {
        let value = rhs.into();
        self.map(|c| c + value)
    }
}

impl<T: Into<Real>> AddAssign<T> for ClockValue {
    fn add_assign(&mut self, rhs: T) {
        let value = rhs.into();
        self.map_mut(|x| *x += value);
    }
}

impl<T: Into<Real>> Sub<T> for ClockValue {
    type Output = ClockValue;
    fn sub(self, rhs: T) -> Self::Output {
        let value = rhs.into();
        self.map(|c| c - value)
    }
}

impl<T: Into<Real>> SubAssign<T> for ClockValue {
    fn sub_assign(&mut self, rhs: T) {
        let value = rhs.into();
        self.map_mut(|x| *x -= value);
    }
}

impl<T: Into<Real>> Mul<T> for ClockValue {
    type Output = ClockValue;
    fn mul(self, rhs: T) -> Self::Output {
        let value = rhs.into();
        self.map(|c| c * value)
    }
}

impl<T: Into<Real>> MulAssign<T> for ClockValue {
    fn mul_assign(&mut self, rhs: T) {
        let value = rhs.into();
        self.map_mut(|x| *x -= value);
    }
}

impl<T: Into<Real>> Div<T> for ClockValue {
    type Output = ClockValue;
    fn div(self, rhs: T) -> Self::Output {
        let value = rhs.into();
        self.map(|c| c / value)
    }
}

impl<T: Into<Real>> DivAssign<T> for ClockValue {
    fn div_assign(&mut self, rhs: T) {
        let value = rhs.into();
        self.map_mut(|x| *x /= value);
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