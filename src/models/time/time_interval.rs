use std::{cmp::{max, min}, fmt, ops::{Add, Sub, Mul}};
use num_traits::One;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{TimeBound, ClockValue};

use TimeBound::*;

/// Time interval with bounds either integer of infinite
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash)]
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
            return ClockValue::disabled();
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

impl Mul for TimeInterval { // Intersection
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

impl fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let first_bound_type = if let Large(_) = self.0 { "[" } else { "]" };
        let second_bound_type = if let Large(_) = self.1 { "]" } else { "[" };
        write!(f, "{}{},{}{}", first_bound_type, self.0, self.1, second_bound_type)
    }
}
