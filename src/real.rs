use std::{fmt::Display, iter::Sum, ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Rem, Sub, SubAssign}};

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

impl Display for Real {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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

impl AddAssign for Real {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl Sub for Real {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Real(self.0 - rhs.0)
    }
}

impl SubAssign for Real {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl Mul for Real {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Real(self.0 * rhs.0)
    }
}

impl MulAssign for Real {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0
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

impl From<Real> for Len {
    fn from(value: Real) -> Self {
        Len::Finite(value)
    }
}

impl Zero for Len {
    fn zero() -> Self {
        Real::zero().into()
    }

    fn is_zero(&self) -> bool {
        match self {
            Len::Finite(r) => r.is_zero(),
            _ => false,
        }
    }
}

impl One for Len {
    fn one() -> Self {
        Real::one().into()
    }
}

impl Add for Len {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Len::Infinite, Len::Infinite) 
            | (Len::Infinite, Len::Finite(_)) 
            | (Len::Finite(_), Len::Infinite) 
                => Len::Infinite,
            (Len::Finite(r1), Len::Finite(r2)) => Len::Finite(r1 + r2),
        }
    }
}

impl Mul for Len {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Len::Infinite, Len::Infinite) 
            | (Len::Infinite, Len::Finite(_)) 
            | (Len::Finite(_), Len::Infinite) 
                => Len::Infinite,
            (Len::Finite(r1), Len::Finite(r2)) => Len::Finite(r1 * r2),
        }
    }
}

impl AddAssign for Len {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (x @ Len::Infinite, Len::Infinite) 
            | (x @ Len::Infinite, Len::Finite(_)) 
            | (x @ Len::Finite(_), Len::Infinite) 
                => *x = Len::Infinite,
            (Len::Finite(r1), Len::Finite(r2)) => *r1 += r2,
        }
    }
}

impl MulAssign for Len {
    fn mul_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (x @ Len::Infinite, Len::Infinite) 
            | (x @ Len::Infinite, Len::Finite(_)) 
            | (x @ Len::Finite(_), Len::Infinite) 
                => *x = Len::Infinite,
            (Len::Finite(r1), Len::Finite(r2)) => *r1 *= r2,
        }
    }
}

impl Sum for Len {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut res = Len::Finite(Real::zero());
        for i in iter {
            res += i;
        }
        res
    }
}