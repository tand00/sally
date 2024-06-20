use std::{marker::PhantomData, ops::{AddAssign, Sub}};

use nalgebra::Scalar;
use num_traits::{Bounded, Zero};

// Either complement or difference MUST be implemented !
pub trait Convex<T : Scalar> : Scalar {

    fn contains(&self, elem : &T) -> bool;

    fn intersection(self, other : Self) -> Self where Self:Sized;

    fn full() -> Self where Self : Sized;

    fn is_empty(&self) -> bool;

    fn union(self, other : Self) -> Disjoint<T,Self> where Self : Sized;

    fn complement(self) -> Disjoint<T,Self> where Self : Sized {
        Self::full().difference(self)
    }

    fn difference(self, other : Self) -> Disjoint<T,Self> where Self : Sized {
        let disj = other.complement();
        disj.intersection(self)
    }

    fn intersects(&self, other : &Self) -> bool where Self : Clone {
        !self.clone().intersection(other.clone()).is_empty()
    }

    fn mut_intersect(&mut self, other : Self) where Self:Sized {
        *self = self.clone().intersection(other)
    }

    fn covers(&self, other : &Self) -> bool {
        other.clone().difference(self.clone()).is_empty()
    }

    fn fuse(set : &mut Vec<Self>, elem : Self) where Self : Sized {
        set.push(elem)
    }

    fn delta(&mut self, dx : T) where T : AddAssign;

}

pub trait Measurable {

    fn len(&self) -> f64;

}

// VERY UNOPTIMIZED FOR NOW !
#[derive(Debug, PartialEq, Clone)]
pub struct Disjoint<T : Scalar, U : Convex<T>> {
    pub intervals : Vec<U>,
    phantom : PhantomData<T>
}

impl<T : Scalar, U : Convex<T>> Disjoint<T,U> {

    pub fn new() -> Self {
        Self { intervals : Vec::new(), phantom : PhantomData }
    }

    pub fn to_convex(mut self) -> Option<U> {
        if self.n_intervals() == 1 {
            self.intervals.pop()
        } else {
            None
        }
    }

    pub fn n_intervals(&self) -> usize {
        self.intervals.len()
    }

    pub fn contains(&self, elem : &T) -> bool {
        for interv in self.intervals.iter() {
            if interv.contains(elem) {
                return true;
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        return self.intervals.is_empty()
    }

    pub fn union(mut self, set : impl Into<Self>) -> Self {
        let disj : Self = set.into();
        if disj.is_empty() {
            return self;
        }
        for interval in disj.intervals {
            U::fuse(&mut self.intervals, interval);
        }
        self
    }

    pub fn difference(mut self, set : impl Into<Self>) -> Self {
        let disj : Self = set.into();
        for interval in disj.intervals {
            let mut index = 0;
            let mut to_add = Vec::new();
            while index < self.intervals.len() {
                let current = &self.intervals[index];
                if !current.intersects(&interval) {
                    index += 1;
                    continue;
                }
                let current = self.intervals.remove(index);
                let mut diff = current.difference(interval.clone());
                if diff.is_empty() {
                    continue;
                }
                to_add.append(&mut diff.intervals);
            } 
            for new_interv in to_add {
                self = self.union(new_interv);
            }
        }
        self
    }

    // Preserves ordering
    pub fn intersection(self, set : impl Into<Self>) -> Self {
        let disj : Self = set.into();
        let mut new_intervals = Vec::new();
        for interval in disj.intervals {
            let mut in_this = Vec::new();
            for this_interv in self.intervals.iter() {
                let inters = interval.clone().intersection(this_interv.clone());
                if inters.is_empty() {
                    continue;
                }
                in_this.push(inters);
            }
            new_intervals.append(&mut in_this);
        }
        Self { intervals : new_intervals, phantom : PhantomData }
    }

    pub fn intersects(&self, set : &Self) -> bool {
        for interval in set.intervals.iter() {
            for this_interv in self.intervals.iter() {
                if interval.intersects(this_interv) {
                    return true;
                }
            }
        }
        false
    }

    pub fn complement(self) -> Self {
        let mut disj : Self = U::full().into();
        for interval in self.intervals {
            disj = disj.difference(interval);
        }
        disj
    }

    pub fn delta(&mut self, dx : T) where T : AddAssign {
        for interval in self.intervals.iter_mut() {
            interval.delta(dx.clone());
        }
    }

}

impl<T : Scalar, U : Convex<T> + Measurable> Measurable for Disjoint<T,U> {
    fn len(&self) -> f64 {
        self.intervals.iter().map(|i| i.len()).sum()
    }
}

impl<T : Scalar, U : Convex<T>> From<U> for Disjoint<T,U> {
    fn from(value: U) -> Self {
        if value.is_empty() {
            return Disjoint::new();
        }
        Disjoint { intervals : vec![value], phantom : PhantomData }
    }
}

impl<T : Scalar, U : Convex<T>> From<Vec<U>> for Disjoint<T,U> {
    fn from(value: Vec<U>) -> Self {
        Disjoint { intervals : value, phantom : PhantomData }
    }
}

impl<T : Scalar, U : Convex<T>> From<(U,U)> for Disjoint<T,U> {
    fn from(value: (U,U)) -> Self {
        Disjoint { intervals : vec![value.0, value.1], phantom : PhantomData }
    }
}

impl<T : Scalar + PartialOrd + Bounded> Convex<T> for (T,T) {

    fn contains(&self, elem : &T) -> bool {
        (self.0 <= *elem) && (*elem <= self.1)
    }

    fn intersection(self, other : Self) -> Self {
        let inter = (
            if other.0 > self.0 { other.0 } else { self.0 },
            if other.1 < self.1 { other.1 } else { self.1 }
        );
        if inter.is_empty() {
            (T::max_value(), T::min_value())
        } else {
            inter
        }
    }

    fn full() -> Self {
        (T::min_value(), T::max_value())
    }

    fn is_empty(&self) -> bool {
        self.1 < self.0
    }

    fn union(self, other : Self) -> Disjoint<T,Self> {
        if self.intersects(&other) {
            return (
                if self.0 < other.0 { self.0 } else { other.0 },
                if self.1 > other.1 { self.1 } else { other.1 }
            ).into();
        }
        (self, other).into()
    }

    fn complement(self) -> Disjoint<T,Self> where Self : Sized {
        if self.0 == T::min_value() && self.1 == T::max_value() {
            (T::max_value(), T::min_value()).into()
        } else if self.0 == T::min_value() {
            (self.1, T::max_value()).into()
        } else if self.1 == T::max_value() {
            (T::min_value(), self.0).into()
        } else {
            (
                (T::min_value(), self.0),
                (self.1, T::max_value())
            ).into()
        }
    }

    fn intersects(&self, other : &Self) -> bool where Self : Clone {
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
            self.0 = T::max_value();
            self.1 = T::min_value();
        }
    }

    fn covers(&self, other : &Self) -> bool {
        self.0 <= other.0 && self.1 >= other.1
    }

    fn delta(&mut self, dx : T)  where T : AddAssign {
        self.0 += dx.clone();
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
        for (i, (a,b)) in set.iter().enumerate() {
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
            (_, false, true) => set[indexs.0] = (elem.0, set[indexs.1].1.clone())
        }
        if diff != 0 {
            for _ in 0..(diff - 1) {
                set.remove(indexs.0 + 1);
            }
        }
    }

}

impl<T : Scalar + PartialOrd + Sub<Output = T> + Into<f64>> Measurable for (T,T) {

    fn len(&self) -> f64 {
        if self.1 < self.0 {
            return f64::zero();
        }
        (self.1.clone() - self.0.clone()).into()
    }

}

pub enum ContinuousSet<T : Scalar + PartialOrd + Bounded, U : Convex<T>> {
    EmptySet,
    ConvexSet(U),
    DisjointSet(Disjoint<T,U>)
}

use ContinuousSet::*;

impl <T : Scalar + PartialOrd + Bounded, U : Convex<T>> ContinuousSet<T,U> {

    pub fn new() -> Self {
        EmptySet
    }

    pub fn contains(&self, elem : &T) -> bool {
        match self {
            EmptySet => false,
            ConvexSet(c) => c.contains(elem),
            DisjointSet(d) => d.contains(elem)
        }
    }

    pub fn is_convex(&self) -> bool {
        match self {
            EmptySet => true,
            ConvexSet(_) => true,
            DisjointSet(_) => false
        }
    }

    pub fn intersection(self, other : impl Into<Self>) -> Self {
        match (self,other.into()) {
            (EmptySet, _) => EmptySet,
            (_, EmptySet) => EmptySet,
            (ConvexSet(c), ConvexSet(c2)) => c.intersection(c2).into(),
            (DisjointSet(d), DisjointSet(d2)) => d.intersection(d2).into(),
            (ConvexSet(c), DisjointSet(d)) | (DisjointSet(d), ConvexSet(c))
                => d.intersection(c).into()
        }
    }

    pub fn full() -> Self {
        ConvexSet(U::full())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            EmptySet => true,
            _ => false
        }
    }

    pub fn union(self, other : impl Into<Self>) -> Self {
        match (self,other.into()) {
            (EmptySet, o) => o,
            (s, EmptySet) => s,
            (ConvexSet(c), ConvexSet(c2)) => c.union(c2).into(),
            (DisjointSet(d), DisjointSet(d2)) => d.union(d2).into(),
            (ConvexSet(c), DisjointSet(d)) | (DisjointSet(d), ConvexSet(c))
                => d.union(c).into()
        }
    }

    pub fn complement(self) -> Self {
        match self {
            EmptySet => Self::full(),
            ConvexSet(c) => c.complement().into(),
            DisjointSet(d) => d.complement().into()
        }
    }

    pub fn intersects(&self, other : &Self) -> bool where Self : Clone {
        match (self,other) {
            (EmptySet, _) => false,
            (_, EmptySet) => false,
            (ConvexSet(c), ConvexSet(c2)) => c.intersects(c2),
            (DisjointSet(d), DisjointSet(d2)) => d.intersects(d2),
            (ConvexSet(c), DisjointSet(d)) | (DisjointSet(d), ConvexSet(c))
                => d.intersects(&c.clone().into()).into()
        }
    }

    pub fn delta(&mut self, dx : T)  where T : AddAssign {
        match self {
            EmptySet => (),
            ConvexSet(c) => c.delta(dx).into(),
            DisjointSet(d) => d.delta(dx).into()
        }
    }

    pub fn convexs(&self) ->  ConvexIterator<T,U> {
        self.into()
    }

}

impl <T : Scalar + PartialOrd + Bounded, U : Convex<T>> From<U> for ContinuousSet<T,U> {
    fn from(value: U) -> Self {
        if value.is_empty() {
            EmptySet
        } else {
            ConvexSet(value)
        }
    }
}

impl <T : Scalar + PartialOrd + Bounded, U : Convex<T>> From<Disjoint<T,U>> for ContinuousSet<T,U> {
    fn from(value: Disjoint<T,U>) -> Self {
        if value.is_empty() {
            EmptySet
        } else if value.n_intervals() == 1 {
            value.to_convex().unwrap().into()
        } else {
            DisjointSet(value)
        }
    }
}

impl <T : Scalar + PartialOrd + Bounded, U : Convex<T>> From<Vec<U>> for ContinuousSet<T,U> {
    fn from(value: Vec<U>) -> Self {
        let d : Disjoint<T,U> = value.into();
        d.into()
    }
}

pub struct ConvexIterator<'a, T : Scalar + PartialOrd + Bounded, U : Convex<T>> {
    set : &'a ContinuousSet<T,U>,
    current : usize
}

impl <'a, T : Scalar + PartialOrd + Bounded, U : Convex<T>> From<&'a ContinuousSet<T,U>> for ConvexIterator<'a,T,U> {
    fn from(value: &'a ContinuousSet<T,U>) -> Self {
        ConvexIterator { set: value, current: 0 }
    }
}

impl<'a, T : Scalar + PartialOrd + Bounded, U : Convex<T>> Iterator for ConvexIterator<'a,T,U> {
    type Item = &'a U;
    fn next(&mut self) -> Option<Self::Item> {
        match self.set {
            EmptySet => None,
            ConvexSet(c) => {
                if self.current == 0 {
                    self.current += 1;
                    Some(c)
                } else {
                    None
                }
            },
            DisjointSet(d) => {
                if self.current < d.intervals.len() {
                    let conv = &d.intervals[self.current];
                    self.current += 1;
                    Some(conv)
                } else {
                    None
                }
            }
        }
    }
}