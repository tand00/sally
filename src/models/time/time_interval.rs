use std::{fmt::{self, Display}, ops::Mul};
use nalgebra::Scalar;
use num_traits::{Bounded, One, Zero};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::computation::intervals::{Convex, Delta, Disjoint, Measurable, ToPositive};

use super::{clock_value::TimeType, Bound, ClockValue, RealTimeBound, TimeBound};

use super::Bound::*;

/// Time interval with bounds either integer of infinite
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Interval<T>(pub Bound<T>, pub Bound<T>);

pub type TimeInterval = Interval<i32>;
pub type RealTimeInterval = Interval<ClockValue>;

impl TimeInterval {

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
        while !self.contains(&chosen) {
            chosen = ClockValue::from(gen.gen_range(low..high)); // If on strict bound
        }
        chosen
    }

    pub fn real(&self) -> RealTimeInterval {
        Interval(self.0.clone().real(), self.1.clone().real())
    }

}

impl<T> Interval<T> {

    pub fn new(a : Bound<T>, b : Bound<T>) -> Self
        where T : TimeType + Scalar + PartialOrd + Bounded
    {
        let res = Interval(a,b);
        if res.is_empty() {
            Self::empty()
        } else {
            res
        }
    }

    pub fn empty() -> Self {
        Interval(Infinite, MinusInfinite)
    }

    pub fn invariant(bound : Bound<T>) -> Self
        where T : Zero + PartialEq
    {
        Interval(Bound::zero(), bound)
    }

}

impl<T : TimeType + Scalar + PartialOrd + Bounded> Mul for Interval<T>  { // Intersection
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl<T : TimeType + Scalar + PartialOrd + Bounded> One for Interval<T> {
    fn one() -> Self {
        Interval::full()
    }
    fn is_one(&self) -> bool {
        self.0 == MinusInfinite && self.1 == Infinite
    }
}

impl<T : TimeType + Scalar + PartialOrd + Bounded> Default for Interval<T> {
    fn default() -> Self {
        Interval::full()
    }
}

impl Display for TimeInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "{{}}");
        }
        let first_bound = match self.0 {
            Infinite => String::from("(inf"),
            MinusInfinite => String::from("(-inf"),
            Large(x) => format!("[{}", x),
            Strict(x) => format!("]{}", x)
        };
        let second_bound = match self.1 {
            Infinite => String::from("inf)"),
            MinusInfinite => String::from("-inf)"),
            Large(x) => format!("{}]", x),
            Strict(x) => format!("{}[", x)
        };
        write!(f, "{},{}", first_bound, second_bound)
    }
}

impl<T : TimeType + Scalar + PartialOrd + Bounded> Convex<ClockValue> for Interval<T> {

    fn contains(&self, elem : &ClockValue) -> bool {
        self.0.lower_than(elem) && self.1.greater_than(elem)
    }

    fn intersection(self, other : Self) -> Self {
        let inter = Interval(
            if (!self.0) > (!other.0) { self.0 } else { other.0 },
            if self.1 < other.1 { self.1 } else { other.1 }
        );
        if inter.is_empty() {
            Interval::empty()
        } else {
            inter
        }
    }

    fn full() -> Self {
        Interval(MinusInfinite, Infinite)
    }

    fn is_empty(&self) -> bool {
        match (self.0, self.1) {
            (Strict(x), Strict(y)) => x >= y,
            _ => self.0 > self.1
        }
    }

    fn union(self, other : Self) -> Disjoint<ClockValue,Self> {
        if self.intersects(&other) {
            return Interval(
                if self.0 < other.0 { self.0 } else { other.0 },
                if self.1 > other.1 { self.1 } else { other.1 }
            ).into();
        }
        (self, other).into()
    }

    fn complement(self) -> Disjoint<ClockValue,Self> {
        if let Interval(MinusInfinite, Infinite) = self {
            Disjoint::new()
        } else if self.0 == MinusInfinite {
            Interval(!self.1, Infinite).into()
        } else if self.1 == Infinite {
            Interval(MinusInfinite, !self.0).into()
        } else {
            (
                Interval(Bound::min_value(), !self.0),
                Interval(!self.1, Bound::max_value())
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
        for (i, Interval(a,b)) in set.iter().enumerate() {
            if indexs.0 != i && indexs.1 != i { break; }
            let lower_skip = match (*b, elem.0) {
                (Large(i), Large(j)) => i < j,
                (Large(i), Strict(j)) => i <= j,
                (Strict(i), Large(j)) => i < j,
                (Strict(i), Strict(j)) => i <= j,
                (MinusInfinite, _) => true,
                _ => false
            };
            let lower_contained = match (*a, elem.0) {
                (Large(i), Large(j)) => i <= j,
                (Large(i), Strict(j)) => i <= j,
                (Strict(i), Large(j)) => i < j,
                (Strict(i), Strict(j)) => i <= j,
                (MinusInfinite, _) => true,
                _ => false
            };
            let upper_contained = match (*a, elem.1) {
                (Large(i), Large(j)) => i <= j,
                (Large(i), Strict(j)) => i <= j,
                (Strict(i), Large(j)) => i <= j,
                (Strict(i), Strict(j)) => i < j,
                (MinusInfinite, _) => true,
                _ => false
            };
            if lower_skip {
                indexs.0 += 1;
            } else if lower_contained {
                contained.0 = true;
            }
            if *b < elem.1 {
                indexs.1 += 1;
            } else if upper_contained {
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
            (_, false, true) => set[indexs.0] = Interval(elem.0, set[indexs.1].1.clone())
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
impl Measurable for RealTimeInterval {
    fn len(&self) -> f64 {
        ClockValue::from(self.1 - self.0).float()
    }
}

impl<T : TimeType + Scalar + PartialOrd + Bounded + Zero> ToPositive for Interval<T> {

    fn positive(self) -> Self {
        self.intersection(Interval(Large(T::zero()), Infinite))
    }

}

impl Delta<TimeBound> for TimeInterval {
    fn delta(&mut self, dx : TimeBound) {
        self.0 += dx;
        self.1 += dx;
    }
}
impl Delta<RealTimeBound> for RealTimeInterval {
    fn delta(&mut self, dx : RealTimeBound) {
        self.0 += dx;
        self.1 += dx;
    }
}
impl Delta<ClockValue> for RealTimeInterval {
    fn delta(&mut self, dx : ClockValue) {
        self.0 += Large(dx);
        self.1 += Large(dx);
    }
}
