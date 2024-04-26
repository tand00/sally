use std::{cmp::{max, min}, fmt, hash::Hash, ops::{Add, AddAssign, Mul, MulAssign, Neg, Not, Sub, SubAssign}};
use num_traits::{Bounded, One, Zero};

/// Integer / Infinite time bound, represents a "</<=" integer constraint
#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum TimeBound {
    Strict(i32),
    Large(i32),
    Infinite,
    MinusInfinite,
}

use rand::Rng;
use TimeBound::{Strict, Large, Infinite, MinusInfinite};

impl TimeBound {
    pub fn greater_than(&self, clock : ClockValue) -> bool {
        match *self {
            Infinite => true,
            Strict(x) => (x as f64) > clock.0,
            Large(x) => (x as f64) >= clock.0,
            MinusInfinite => false,
        }
    }
    pub fn lower_than(&self, clock : ClockValue) -> bool {
        match *self {
            Infinite => false,
            Strict(x) => (x as f64) < clock.0,
            Large(x) => (x as f64) <= clock.0,
            MinusInfinite => true,
        }
    }
    pub fn intersection(self, other : TimeBound) -> TimeBound {
        min(self, other)
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

/// Time interval with bounds either integer of infinite
#[derive(Debug,Copy,Clone)]
pub struct TimeInterval(pub TimeBound, pub TimeBound);

impl TimeInterval {
    pub fn contains(&self, clock : ClockValue) -> bool {
        self.0.lower_than(clock) && self.1.greater_than(clock)
    }
    pub fn full() -> TimeInterval {
        TimeInterval(MinusInfinite, Infinite)
    }
    pub fn delta(self, bound : TimeBound) -> TimeInterval {
        TimeInterval(self.0 - bound, self.1 - bound)
    }
    pub fn is_empty(&self) -> bool {
        match (self.0, self.1) {
            (Strict(x), Strict(y)) => x >= y,
            _ => self.0 > self.1
        }
    }
    pub fn random_date(&self) -> ClockValue {
        let mut gen = rand::thread_rng();
        if self.is_empty() {
            panic!("Interval is empty !");
        }
        let low = match self.0 {
            Infinite => f64::NAN,
            Large(x) | Strict(x) => x as f64,
            MinusInfinite => f64::NEG_INFINITY
        };
        let high = match self.1 {
            MinusInfinite => f64::NAN,
            Large(x) | Strict(x) => x as f64,
            Infinite => f64::INFINITY
        };
        let mut chosen = ClockValue(gen.gen_range(low..high));
        while !self.contains(chosen) {
            chosen = ClockValue(gen.gen_range(low..high)); // If on strict bound
        }
        chosen
    }
}

impl Mul for TimeInterval {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        TimeInterval(max(self.0, rhs.0), min(self.1, rhs.1))
    }
}

impl One for TimeInterval {
    fn one() -> Self {
        TimeInterval::full()
    }
    fn is_one(&self) -> bool {
        self.0 == MinusInfinite && self.1 == Infinite
    }
}

// Wrapper for f64 to implement extern traits
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
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


// Display implementations ---

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

impl fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let first_bound_type = if let Large(_) = self.0 { "[" } else { "]" };
        let second_bound_type = if let Large(_) = self.1 { "]" } else { "[" };
        write!(f, "{}{},{}{}", first_bound_type, self.0, self.1, second_bound_type)
    }
}
