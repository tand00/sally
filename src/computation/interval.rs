use std::{fmt::{self, Display}, ops::{Add, AddAssign, BitAnd, Mul, Neg, Sub, SubAssign}};
use num_traits::{Bounded, Zero};
use rand_distr::uniform::SampleRange;
use serde::{Deserialize, Serialize};

use crate::{computation::convex::{Convex, Delta, Disjoint, Measurable, ToPositive}, real::{Len, Real}};

use super::Bound;

use super::Bound::*;

/// Time interval with bounds either integer of infinite
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Interval<T>(pub Bound<T>, pub Bound<T>);

fn check_emptiness<T: PartialOrd>(interval: &Interval<T>) -> bool {
    match (&interval.0, &interval.1) {
        (Strict(x), Strict(y)) => x >= y,
        _ => interval.0 > interval.1
    }
}

impl<T> Interval<T> {

    pub fn new(a : Bound<T>, b : Bound<T>) -> Self 
        where T: PartialOrd
    {
        let res = Interval(a,b);
        if res.is_empty() {
            Self::empty()
        } else {
            res
        }
    }

    pub const fn empty() -> Self {
        Interval(Infinite, MinusInfinite)
    }

    pub fn lower(&self) -> &Bound<T> {
        &self.0
    }

    pub fn upper(&self) -> &Bound<T> {
        &self.1
    }

    pub fn lower_mut(&mut self) -> &mut Bound<T> {
        &mut self.0
    }

    pub fn upper_mut(&mut self) -> &mut Bound<T> {
        &mut self.1
    }

    pub fn is_empty(&self) -> bool 
        where T: PartialOrd
    {
        check_emptiness(self)
    }

    pub fn cast<U: From<T>>(self) -> Interval<U> {
        Interval(self.0.cast(), self.1.cast())
    }

    pub fn open(self) -> Interval<T> {
        Interval(self.0.strict(), self.1.strict())
    }

    pub fn make_open(&mut self) {
        self.0.make_strict();
        self.1.make_strict();
    }

    pub fn closed(self) -> Interval<T> {
        Interval(self.0.large(), self.1.large())
    }

    pub fn make_closed(&mut self) {
        self.0.make_large();
        self.1.make_large();
    }

}

impl<T : Add<Output = T>> Add for Interval<T> { 
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Interval(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T : Add<Output = T>> AddAssign for Interval<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl<T : Add<Output = T> + Clone> Add<T> for Interval<T> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Interval(self.0 + rhs.clone(), self.1 + rhs)
    }
}

impl<T : Add<Output = T> + Clone> AddAssign<T> for Interval<T> { 
    fn add_assign(&mut self, rhs: T) {
        self.0 += rhs.clone();
        self.1 += rhs;
    }
}

impl<T : Sub<Output = T>> Sub for Interval<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Interval(self.0 - rhs.1, self.1 - rhs.0)
    }
}

impl<T : Sub<Output = T>> SubAssign for Interval<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
    }
}

impl<T : Sub<Output = T> + Clone> Sub<T> for Interval<T> {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Interval(self.0 - rhs.clone(), self.1 - rhs)
    }
}

impl<T : Sub<Output = T> + Clone> SubAssign<T> for Interval<T> { // Intersection
    fn sub_assign(&mut self, rhs: T) {
        self.0 -= rhs.clone();
        self.1 -= rhs;
    }
}

impl<T : PartialOrd + Bounded + Clone> BitAnd for Interval<T> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl<T : PartialOrd + Mul<Output = T> + Zero + Clone> Mul<T> for Interval<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        if rhs < T::zero() {
            Interval(self.1 * rhs.clone(), self.0 * rhs)
        } else {
            Interval(self.0 * rhs.clone(), self.1 * rhs)
        }
    }
}

impl<T : Neg<Output = T>> Neg for Interval<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Interval(-self.1, -self.0)
    }
}

impl<T> Default for Interval<T> {
    fn default() -> Self {
        Interval(MinusInfinite, Infinite)
    }
}

impl<T: Display + PartialOrd> Display for Interval<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "{{}}");
        }
        let first_bound = match &self.0 {
            Infinite => String::from("(inf"),
            MinusInfinite => String::from("(-inf"),
            Large(x) => format!("[{}", x),
            Strict(x) => format!("]{}", x)
        };
        let second_bound = match &self.1 {
            Infinite => String::from("inf)"),
            MinusInfinite => String::from("-inf)"),
            Large(x) => format!("{}]", x),
            Strict(x) => format!("{}[", x)
        };
        write!(f, "{},{}", first_bound, second_bound)
    }
}

impl<T : Clone + PartialOrd> Convex<T> for Interval<T> {

    fn contains(&self, elem : &T) -> bool {
        self.0.lower_than(elem) && self.1.greater_than(elem)
    }

    fn intersection(self, other : Self) -> Self {
        let inter = Interval(
            if (!self.0.clone()) > (!other.0.clone()) { self.0 } else { other.0 },
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
        check_emptiness(self)
    }

    fn union(self, other : Self) -> Disjoint<T,Self> {
        if self.intersects(&other) {
            return Interval(
                if self.0 < other.0 { self.0 } else { other.0 },
                if self.1 > other.1 { self.1 } else { other.1 }
            ).into();
        }
        (self, other).into()
    }

    fn complement(self) -> Disjoint<T,Self> {
        if let Interval(MinusInfinite, Infinite) = self {
            Disjoint::new()
        } else if self.0 == MinusInfinite {
            Interval(!self.1, Infinite).into()
        } else if self.1 == Infinite {
            Interval(MinusInfinite, !self.0).into()
        } else {
            (
                Interval(Bound::MinusInfinite, !self.0),
                Interval(!self.1, Bound::Infinite)
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
            let lower_skip = match (b, &elem.0) {
                (Large(i), Large(j)) => i < j,
                (Large(i), Strict(j)) => i <= j,
                (Strict(i), Large(j)) => i < j,
                (Strict(i), Strict(j)) => i <= j,
                (MinusInfinite, _) => true,
                _ => false
            };
            let lower_contained = match (a, &elem.0) {
                (Large(i), Large(j)) => i <= j,
                (Large(i), Strict(j)) => i <= j,
                (Strict(i), Large(j)) => i < j,
                (Strict(i), Strict(j)) => i <= j,
                (MinusInfinite, _) => true,
                _ => false
            };
            let upper_contained = match (a, &elem.1) {
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

impl<T: PartialOrd> SampleRange<T> for Interval<T> {
    fn sample_single<R: rand::prelude::Rng + ?Sized>(self, rng: &mut R) -> Result<T, rand_distr::uniform::Error> {
        todo!()
    }

    fn is_empty(&self) -> bool {
        check_emptiness(self)
    }
}

impl<T: Zero + PartialEq> From<Bound<T>> for Interval<T> {
    fn from(bound : Bound<T>) -> Self {
        Interval(Bound::zero(), bound)
    }
}

impl Measurable for Interval<Real> {
    fn len(&self) -> Len {
        match self.1 - self.0 {
            Strict(x) | Large(x) => x.into(),
            _ => Len::Infinite
        }
    }
}

impl Measurable for Interval<i32> {
    fn len(&self) -> Len {
        match self.1 - self.0 {
            Strict(x) | Large(x) => Real::from(x).into(),
            _ => Len::Infinite
        }
    }
}

impl<T : Clone + PartialOrd + Zero> ToPositive for Interval<T> {
    fn positive(self) -> Self {
        self.intersection(Interval(Large(T::zero()), Infinite))
    }
}

impl<T: Add<Output = T> + Clone> Delta<T> for Interval<T> {
    fn delta(&mut self, dx: T) {
        self.0 += dx.clone();
        self.1 += dx;
    }
}