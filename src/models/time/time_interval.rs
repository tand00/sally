use std::{cmp::{max, min}, fmt, ops::Mul};
use num_traits::{Bounded, One};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::computation::intervals::{Convex, Disjoint, Measurable};

use super::{TimeBound, ClockValue};

use TimeBound::*;

/// Time interval with bounds either integer of infinite
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct TimeInterval(pub TimeBound, pub TimeBound);

impl TimeInterval {
    pub fn contains(&self, clock : ClockValue) -> bool {
        self.0.lower_than(clock) && self.1.greater_than(clock)
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
        let mut chosen = ClockValue::from(gen.gen_range(low..high));
        while !self.contains(chosen) {
            chosen = ClockValue::from(gen.gen_range(low..high)); // If on strict bound
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

impl Default for TimeInterval {
    fn default() -> Self {
        TimeInterval::full()
    }
}

impl fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let first_bound_type = if let Large(_) = self.0 { "[" } else { "]" };
        let second_bound_type = if let Large(_) = self.1 { "]" } else { "[" };
        write!(f, "{}{},{}{}", first_bound_type, self.0, self.1, second_bound_type)
    }
}

impl Convex<TimeBound> for TimeInterval {

    fn contains(&self, elem : &TimeBound) -> bool {
        (self.0 <= *elem) && (*elem <= self.1)
    }

    fn intersection(self, other : Self) -> Self {
        let inter = TimeInterval(
            if other.0 > self.0 { other.0 } else { self.0 },
            if other.1 < self.1 { other.1 } else { self.1 }
        );
        if inter.is_empty() {
            TimeInterval(Infinite, MinusInfinite)
        } else {
            inter
        }
    }

    fn full() -> Self {
        TimeInterval(MinusInfinite, Infinite)
    }

    fn is_empty(&self) -> bool {
        match (self.0, self.1) {
            (Strict(x), Strict(y)) => x >= y,
            _ => self.0 > self.1
        }
    }

    fn union(self, other : Self) -> Disjoint<TimeBound,Self> {
        if self.intersects(&other) {
            return TimeInterval(
                if self.0 < other.0 { self.0 } else { other.0 },
                if self.1 > other.1 { self.1 } else { other.1 }
            ).into();
        }
        (self, other).into()
    }

    fn complement(self) -> Disjoint<TimeBound,Self> {
        if let TimeInterval(MinusInfinite, Infinite) = self {
            TimeInterval(Infinite, MinusInfinite).into()
        } else if self.0 == MinusInfinite {
            TimeInterval(!self.1, Infinite).into()
        } else if self.1 == Infinite {
            TimeInterval(MinusInfinite, !self.0).into()
        } else {
            (
                TimeInterval(TimeBound::min_value(), !self.0),
                TimeInterval(!self.1, TimeBound::max_value())
            ).into()
        }
    }

    fn intersects(&self, other : &Self) -> bool {
        (other.0 <= self.1) && (other.1 >= self.0)
    }

    fn mut_intersect(&mut self, other : Self) {
        if other.0 > self.0 {
            self.0 = other.0
        }
        if other.1 < self.1 {
            self.1 = other.1
        }
        if self.is_empty() {
            self.0 = Infinite;
            self.1 = MinusInfinite;
        }
    }

    fn covers(&self, other : &Self) -> bool {
        self.0 <= other.0 && self.1 >= other.1
    }

    fn delta(&mut self, dx : TimeBound) {
        self.0 += dx;
        self.1 += dx;
    }

    fn fuse(set : &mut Vec<Self>, elem : Self) {
        if elem.is_empty() {
            return;
        }
        if set.is_empty() {
            set.push(elem);
            return;
        }
        let mut indexs = (0, 0);
        let mut contained = (false, false);
        for (i, TimeInterval(a,b)) in set.iter().enumerate() {
            if indexs.0 != i && indexs.1 != i { break; }
            if *b < elem.0 {
                indexs.0 += 1;
            } else if *a <= elem.0 {
                contained.0 = true;
            }
            if *b < elem.1 {
                indexs.1 += 1;
            } else if *a <= elem.1 {
                contained.1 = true;
            }
        }
        let diff = indexs.1 - indexs.0;
        match (indexs.1 - indexs.0, contained.0, contained.1) {
            (0, true, true) => (),
            (0, false, false) => set.insert(indexs.0, elem),
            (_, true, true) => set[indexs.0].1 = set[indexs.1].1.clone(),
            (_, false, false) => set[indexs.0] = elem,
            (_, true, false) => set[indexs.0].1 = elem.1,
            (_, false, true) => set[indexs.0] = TimeInterval(elem.0, set[indexs.1].1.clone())
        }
        if diff != 0 {
            for _ in 0..(diff - 1) {
                set.remove(indexs.0 + 1);
            }
        }
    }

}

impl Measurable for TimeInterval {

    fn len(&self) -> f64 {
        ClockValue::from(self.1 - self.0).float()
    }

}