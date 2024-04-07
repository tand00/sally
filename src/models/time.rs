use std::{cmp::min, fmt, ops::{Add, Neg, Sub}};

/// Integer / Infinite time bound, used to define time intervals
#[derive(Debug,Copy,Clone,PartialEq,Eq,Ord)]
pub enum TimeBound {
    Strict(i32),
    Large(i32),
    Infinite,
    MinusInfinite,
}

use TimeBound::{Strict, Large, Infinite, MinusInfinite};

impl TimeBound {
    pub fn greater_than(&self, clock : f64) -> bool {
        match *self {
            Infinite => true,
            Strict(x) => (x as f64) > clock,
            Large(x) => (x as f64) >= clock,
            MinusInfinite => false,
        }
    }
    pub fn lower_than(&self, clock : f64) -> bool {
        match *self {
            Infinite => false,
            Strict(x) => (x as f64) < clock,
            Large(x) => (x as f64) <= clock,
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

impl Sub for TimeBound {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
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

/// Time interval with bounds either integer of infinite
#[derive(Debug,Copy,Clone)]
pub struct TimeInterval(pub TimeBound, pub TimeBound);

impl TimeInterval {
    pub fn contains(&self, clock : f64) -> bool {
        self.0.lower_than(clock) && self.1.greater_than(clock)
    }
    pub fn full() -> TimeInterval {
        TimeInterval(MinusInfinite, Infinite)
    }
}

// Display implementations ---

impl fmt::Display for TimeBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_print = match self {
            Infinite => "INF".to_string(),
            Strict(i) => i.to_string(),
            Large(i) => i.to_string(),
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
