use std::fmt;

/// Integer / Infinite time bound, used to define time intervals
#[derive(Debug,Copy,Clone)]
pub enum TimeBound {
    Strict(i32),
    Large(i32),
    Infinite,
    MinusInfinite,
}

impl TimeBound {
    pub fn greater_than(&self, clock : f64) -> bool {
        match *self {
            Self::Infinite => true,
            Self::Strict(x) => (x as f64) > clock,
            Self::Large(x) => (x as f64) >= clock,
            Self::MinusInfinite => false,
        }
    }
    pub fn lower_than(&self, clock : f64) -> bool {
        match *self {
            Self::Infinite => false,
            Self::Strict(x) => (x as f64) < clock,
            Self::Large(x) => (x as f64) <= clock,
            Self::MinusInfinite => true,
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
        TimeInterval(TimeBound::MinusInfinite, TimeBound::Infinite)
    }
}

// Display implementations ---

impl fmt::Display for TimeBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_print = match self {
            TimeBound::Infinite => "INF".to_string(),
            TimeBound::Strict(i) => i.to_string(),
            TimeBound::Large(i) => i.to_string(),
            TimeBound::MinusInfinite => "-INF".to_string(),
        };
        write!(f, "{}", to_print)
    }
}

impl fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let first_bound_type = if let TimeBound::Large(_) = self.0 { "[" } else { "]" };
        let second_bound_type = if let TimeBound::Large(_) = self.1 { "]" } else { "[" };
        write!(f, "{}{},{}{}", first_bound_type, self.0, self.1, second_bound_type)
    }
}
