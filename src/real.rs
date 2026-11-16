use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use fixed::{traits::{FromFixed, ToFixed}, types::I16F48};
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};

type InternalType = I16F48;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Real(InternalType);

impl Real {

    pub fn value<T: FromFixed>(self) -> T {
        self.0.to_num()
    }

    pub fn round(self) -> Self {
        Real(self.0.round())
    }

    pub fn abs(self) -> Self {
        Real(self.0.abs())
    }

    pub fn inner(&mut self) -> &mut InternalType {
        &mut self.0
    } 

}

impl<T: ToFixed> From<T> for Real {
    fn from(value: T) -> Self {
        Real(InternalType::from_num(value))
    }
}

impl Add for Real {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Real(self.0 + rhs.0)
    }
}

impl Sub for Real {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Real(self.0 - rhs.0)
    }
}

impl Mul for Real {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Real(self.0 * rhs.0)
    }
}

impl Div for Real {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Real(self.0 / rhs.0)
    }
}

impl Rem for Real {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Real(self.0 % rhs.0)
    }
}

impl Neg for Real {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Real(-self.0)
    }
}

impl Zero for Real {
    fn zero() -> Self {
        Real(InternalType::ZERO)
    }

    fn is_zero(&self) -> bool {
        self.0 == InternalType::ZERO
    }
}

impl One for Real {
    fn one() -> Self {
        Real(InternalType::ONE)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Len {
    Infinite,
    Finite(Real)
}