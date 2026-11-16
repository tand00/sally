use std::{fmt, hash::Hash, ops::{Add, AddAssign, Mul, MulAssign, Neg, Not, Sub, SubAssign}};
use num_traits::{Bounded, One, Zero};
use serde::{Deserialize, Serialize};
use crate::real::Real;

use Bound::{Strict, Large, Infinite, MinusInfinite};

/// Integer / Infinite time bound, represents a "</<=" integer constraint
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
pub enum Bound<T> {
    #[serde(rename="<")]
    Strict(T),
    #[serde(rename="<=")]
    Large(T),
    #[serde(rename="+inf")]
    #[default]
    Infinite,
    #[serde(rename="-inf")]
    MinusInfinite,
}

pub type TimeBound = Bound<Real>;

impl<T: Zero> Bound<T> {
    pub fn zero() -> Self {
        Bound::Large(T::zero())
    }
}

impl<T: One> Bound<T> {
    pub fn one() -> Self {
        Bound::Large(T::one())
    }
}

impl<T : PartialOrd> Bound<T> {
    pub fn greater_than(&self, y : &T) -> bool {
        match self {
            Infinite => true,
            Strict(x) => *x > *y,
            Large(x) => *x >= *y,
            MinusInfinite => false,
        }
    }
    pub fn lower_than(&self, y : &T) -> bool {
        match self {
            Infinite => false,
            Strict(x) => *x < *y,
            Large(x) => *x <= *y,
            MinusInfinite => true,
        }
    }
}

impl<T> Bound<T> {

    pub fn intersection(self, other : Self) -> Self
        where T : PartialOrd
    {
        if self <= other {
            self
        } else {
            other
        }
    }

    pub fn unit() -> Self
        where T : One
    {
        Large(T::one())
    }

}

impl<T : Neg<Output = T>> Neg for Bound<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Infinite => MinusInfinite,
            Strict(x) => Strict(-x),
            Large(x) => Large(-x),
            MinusInfinite => Infinite
        }
    }
}

impl<T : Add<Output = T>> Add for Bound<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Infinite, MinusInfinite) |
                (MinusInfinite, Infinite) => panic!("Indeterminate sum !"),
            (Infinite, _) |
                (_, Infinite) => Infinite,
            (MinusInfinite, _) |
                (_, MinusInfinite) => MinusInfinite,
            (Large(x), Strict(y)) |
                (Strict(x), Large(y)) |
                (Strict(x), Strict(y)) => Strict(x + y),
            (Large(x), Large(y)) => Large(x + y)
        }
    }
}

impl<T : Sub<Output = T>> Sub for Bound<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Infinite, Infinite) |
                (MinusInfinite, MinusInfinite) => panic!("Indeterminate sum !"),
            (MinusInfinite, _) | (_, Infinite) => MinusInfinite,
            (Infinite, _) | (_, MinusInfinite) => Infinite,
            (Large(x), Strict(y)) |
                (Strict(x), Large(y)) |
                (Strict(x), Strict(y)) => Strict(x - y),
            (Large(x), Large(y)) => Large(x - y)
        }
    }
}

impl<T : Add<Output = T>> AddAssign for Bound<T> {
    fn add_assign(&mut self, rhs: Self) {
        let x = std::mem::take(self);
        *self = x + rhs
    }
}

impl<T : Sub<Output = T>> SubAssign for Bound<T> {
    fn sub_assign(&mut self, rhs: Self) {
        let x = std::mem::take(self);
        *self = x - rhs
    }
}

impl<T : PartialOrd> Mul for Bound<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl<T : PartialOrd + Mul<Output = T> + Zero> Mul<T> for Bound<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        match self {
            Strict(x) => Strict(x * rhs),
            Large(x) => Large(x * rhs),
            Infinite => if rhs.is_zero() {
                Large(T::zero())
            } else if rhs < T::zero() {
                MinusInfinite
            } else {
                Infinite
            },
            MinusInfinite => if rhs.is_zero() {
                Large(T::zero())
            } else if rhs < T::zero() {
                Infinite
            } else {
                MinusInfinite
            },
        }
    }
}

impl<T : PartialOrd + PartialEq> MulAssign for Bound<T> {
    fn mul_assign(&mut self, rhs: Self) {
        let temp = std::mem::replace(self, Self::Infinite);
        *self = temp * rhs;
    }
}

impl<T> Not for Bound<T> {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Large(x) => Strict(x),
            Strict(x) => Large(x),
            _ => self
        }
    }
}

impl<T : PartialOrd> PartialOrd for Bound<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Infinite,Infinite) |
                (MinusInfinite,MinusInfinite) => Some(std::cmp::Ordering::Equal),
            (Infinite, _) |
                (_, MinusInfinite) => Some(std::cmp::Ordering::Greater),
            (_, Infinite) |
                (MinusInfinite, _) => Some(std::cmp::Ordering::Less),
            (Strict(x), Strict(y)) => x.partial_cmp(y),
            (Large(x), Large(y)) => x.partial_cmp(y),
            (Strict(x), Large(y)) =>
                if *x > *y {
                    Some(std::cmp::Ordering::Greater)
                } else {
                    Some(std::cmp::Ordering::Less)
                }
            (Large(x), Strict(y)) =>
                if *x >= *y {
                    Some(std::cmp::Ordering::Greater)
                } else {
                    Some(std::cmp::Ordering::Less)
                }
        }
    }
}

impl<T : Ord> Ord for Bound<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<T : Bounded> Bounded for Bound<T> {
    fn max_value() -> Self {
        Infinite
    }
    fn min_value() -> Self {
        MinusInfinite
    }
}

impl<T : fmt::Display> fmt::Display for Bound<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_print = match self {
            Infinite => "INF".to_string(),
            Strict(i) => format!("<{}", i),
            Large(i) => format!("<={}", i),
            MinusInfinite => "-INF".to_string(),
        };
        write!(f, "{}", to_print)
    }
}


impl AddAssign<Real> for TimeBound {
    fn add_assign(&mut self, rhs: Real) {
        *self += Large(rhs)
    }
}

impl SubAssign<Real> for TimeBound {
    fn sub_assign(&mut self, rhs: Real) {
        *self -= Large(rhs)
    }
}

impl Add<Real> for TimeBound {
    type Output = Self;
    fn add(self, rhs: Real) -> Self::Output {
        self + Large(rhs)
    }
}

impl Sub<Real> for TimeBound {
    type Output = Self;
    fn sub(self, rhs: Real) -> Self::Output {
        self - Large(rhs)
    }
}