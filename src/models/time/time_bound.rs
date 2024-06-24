use std::{cmp::min, fmt, hash::Hash, ops::{Add, AddAssign, Mul, MulAssign, Neg, Not, Sub, SubAssign}};
use num_traits::{Bounded, One, Zero};
use serde::{Deserialize, Serialize};
use super::ClockValue;

use TimeBound::{Strict, Large, Infinite, MinusInfinite};

/// Integer / Infinite time bound, represents a "</<=" integer constraint
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TimeBound {
    #[serde(rename="<")]
    Strict(i32),
    #[serde(rename="<=")]
    Large(i32),
    #[serde(rename="+inf")]
    Infinite,
    #[serde(rename="-inf")]
    MinusInfinite,
}

impl TimeBound {
    pub fn greater_than(&self, clock : &ClockValue) -> bool {
        match self {
            Infinite => true,
            Strict(x) => (*x as f64) > clock.float(),
            Large(x) => (*x as f64) >= clock.float(),
            MinusInfinite => false,
        }
    }
    pub fn lower_than(&self, clock : &ClockValue) -> bool {
        match self {
            Infinite => false,
            Strict(x) => (*x as f64) < clock.float(),
            Large(x) => (*x as f64) <= clock.float(),
            MinusInfinite => true,
        }
    }
    pub fn intersection(self, other : TimeBound) -> TimeBound {
        min(self, other)
    }
    pub fn unit() -> TimeBound {
        Large(1)
    }
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
}

impl Neg for TimeBound {
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

impl Add for TimeBound {
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

impl AddAssign for TimeBound {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for TimeBound {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl SubAssign for TimeBound {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul for TimeBound {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl MulAssign for TimeBound {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Not for TimeBound {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Large(x) => Strict(x),
            Strict(x) => Large(x),
            _ => self
        }
    }
}

impl PartialOrd for TimeBound {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (*self, *other) {
            (Infinite,Infinite) |
                (MinusInfinite,MinusInfinite) => Some(std::cmp::Ordering::Equal),
            (Infinite, _) | 
                (_, MinusInfinite) => Some(std::cmp::Ordering::Greater),
            (_, Infinite) | 
                (MinusInfinite, _) => Some(std::cmp::Ordering::Less),
            (Strict(x), Strict(y)) => x.partial_cmp(&y),
            (Large(x), Large(y)) => x.partial_cmp(&y),
            (Strict(x), Large(y)) => 
                if x > y {
                    Some(std::cmp::Ordering::Greater)
                } else {
                    Some(std::cmp::Ordering::Less)
                }
            (Large(x), Strict(y)) => 
                if x >= y {
                    Some(std::cmp::Ordering::Greater)
                } else {
                    Some(std::cmp::Ordering::Less)
                }
        }
    }
}

impl Ord for TimeBound {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Zero for TimeBound {
    fn zero() -> Self {
        Large(0)
    }
    fn is_zero(&self) -> bool {
        *self == Large(0)
    }
}

impl Bounded for TimeBound {
    fn max_value() -> Self {
        Infinite
    }
    fn min_value() -> Self {
        MinusInfinite
    }
}

impl One for TimeBound {
    fn one() -> Self {
        Infinite
    }
    fn is_one(&self) -> bool {
        *self == Infinite
    }
}

impl fmt::Display for TimeBound {
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