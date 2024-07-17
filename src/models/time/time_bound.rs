use std::{fmt, hash::Hash, ops::{Add, AddAssign, Mul, MulAssign, Neg, Not, Sub, SubAssign}};
use num_traits::{Bounded, One, Zero};
use serde::{Deserialize, Serialize};
use super::{clock_value::TimeType, ClockValue};

use Bound::{Strict, Large, Infinite, MinusInfinite};

/// Integer / Infinite time bound, represents a "</<=" integer constraint
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Bound<T> {
    #[serde(rename="<")]
    Strict(T),
    #[serde(rename="<=")]
    Large(T),
    #[serde(rename="+inf")]
    Infinite,
    #[serde(rename="-inf")]
    MinusInfinite,
}

pub type TimeBound = Bound<i32>;
pub type RealTimeBound = Bound<ClockValue>;

impl TimeBound {
    pub fn float(&self) -> f64 {
        match self {
            Infinite => f64::INFINITY,
            MinusInfinite => f64::NEG_INFINITY,
            Strict(x) | Large(x) => *x as f64
        }
    }
    pub fn value(&self) -> i32 {
        match self {
            Infinite => i32::MAX,
            MinusInfinite => i32::MIN,
            Strict(x) | Large(x) => *x
        }
    }
    pub fn real(&self) -> RealTimeBound {
        match self {
            Infinite => Infinite,
            MinusInfinite => MinusInfinite,
            Strict(x) => Strict((*x as f64).into()),
            Large(x) => Large((*x as f64).into())
        }
    }
}

impl<T : TimeType> Bound<T> {
    pub fn greater_than(&self, clock : &ClockValue) -> bool {
        match self {
            Infinite => true,
            Strict(x) => (*x).into() > *clock,
            Large(x) => (*x).into() >= *clock,
            MinusInfinite => false,
        }
    }
    pub fn lower_than(&self, clock : &ClockValue) -> bool {
        match self {
            Infinite => false,
            Strict(x) => (*x).into() < *clock,
            Large(x) => (*x).into() <= *clock,
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

impl<T : Add<Output = T> + TimeType> AddAssign for Bound<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T : Neg<Output = T> + Add<Output = T>> Sub for Bound<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl<T : Neg<Output = T> + Add<Output = T> + TimeType> SubAssign for Bound<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl<T : PartialOrd> Mul for Bound<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl<T : PartialOrd + PartialEq + TimeType> MulAssign for Bound<T> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
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

impl<T : Zero + PartialEq> Zero for Bound<T> {
    fn zero() -> Self {
        Large(T::zero())
    }
    fn is_zero(&self) -> bool {
        *self == Large(T::zero())
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

impl<T : PartialEq + PartialOrd> One for Bound<T> {
    fn one() -> Self {
        Infinite
    }
    fn is_one(&self) -> bool {
        *self == Infinite
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

impl Default for TimeBound {
    fn default() -> Self {
        Infinite
    }
}


impl AddAssign<ClockValue> for RealTimeBound {
    fn add_assign(&mut self, rhs: ClockValue) {
        *self += Large(rhs)
    }
}

impl SubAssign<ClockValue> for RealTimeBound {
    fn sub_assign(&mut self, rhs: ClockValue) {
        *self -= Large(rhs)
    }
}

impl Add<ClockValue> for RealTimeBound {
    type Output = Self;
    fn add(self, rhs: ClockValue) -> Self::Output {
        self + Large(rhs)
    }
}

impl Sub<ClockValue> for RealTimeBound {
    type Output = Self;
    fn sub(self, rhs: ClockValue) -> Self::Output {
        self - Large(rhs)
    }
}