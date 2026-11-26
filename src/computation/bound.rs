use std::{fmt, hash::Hash, ops::{Add, AddAssign, Mul, MulAssign, Neg, Not, Sub, SubAssign}};
use num_traits::{Bounded, One, Zero};
use serde::{Deserialize, Serialize};

use Bound::{Strict, Large, Infinite, MinusInfinite};

use crate::computation::convex::Delta;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
pub enum Bound<T> {
    #[serde(rename="<")]
    Strict(T),
    #[serde(rename="<=")]
    Large(T),
    #[serde(rename="+inf")]
    #[default]
    Infinite,
    #[serde(rename="-inf")]
    MinusInfinite,
}

impl<T: Zero + PartialEq> Zero for Bound<T> {
    
    fn zero() -> Self {
        Bound::Large(T::zero())
    }
    
    fn is_zero(&self) -> bool {
        if let Self::Large(b) = self {
            return *b == T::zero();
        }
        false
    }

}

impl<T : PartialOrd> Bound<T> {

    pub fn greater_than(&self, y : &T) -> bool {
        match self {
            Infinite => true,
            Strict(x) => *x > *y,
            Large(x) => *x >= *y,
            MinusInfinite => false,
        }
    }

    pub fn lower_than(&self, y : &T) -> bool {
        match self {
            Infinite => false,
            Strict(x) => *x < *y,
            Large(x) => *x <= *y,
            MinusInfinite => true,
        }
    }

    pub fn intersection(self, other : Self) -> Self {
        if self <= other {
            self
        } else {
            other
        }
    }

    pub fn mut_intersection(&mut self, other : Self) {
        if *self > other {
            *self = other
        } 
    }

}

impl<T> Bound<T> {

    pub fn unit() -> Self
        where T : One
    {
        Large(T::one())
    }

    pub fn cast<U: From<T>>(self) -> Bound<U> {
        match self {
            Strict(x) => Strict(x.into()),
            Large(x) => Large(x.into()),
            Infinite => Infinite,
            MinusInfinite => MinusInfinite,
        }
    }

    pub fn strict(self) -> Bound<T> {
        match self {
            Large(x) => Strict(x),
            _ => self
        }
    }

    pub fn large(self) -> Bound<T> {
        match self {
            Strict(x) => Large(x),
            _ => self
        }
    }

    pub fn make_strict(&mut self) {
        let temp = std::mem::take(self);
        *self = temp.strict();
    }

    pub fn make_large(&mut self) {
        let temp = std::mem::take(self);
        *self = temp.large();
    }

}

impl<T> From<T> for Bound<T> {
    fn from(value: T) -> Self {
        Bound::Large(value)
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
            (Infinite, MinusInfinite) 
            | (MinusInfinite, Infinite) => panic!("Indeterminate sum !"),
            (Infinite, _) 
            | (_, Infinite) => Infinite,
            (MinusInfinite, _) 
            | (_, MinusInfinite) => MinusInfinite,
            (Large(x), Strict(y)) 
            | (Strict(x), Large(y)) 
            | (Strict(x), Strict(y)) => Strict(x + y),
            (Large(x), Large(y)) => Large(x + y)
        }
    }
}

impl<T : Sub<Output = T>> Sub for Bound<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Infinite, Infinite) 
            | (MinusInfinite, MinusInfinite) => panic!("Indeterminate sum !"),
            (MinusInfinite, _) | (_, Infinite) => MinusInfinite,
            (Infinite, _) | (_, MinusInfinite) => Infinite,
            (Large(x), Strict(y)) 
            | (Strict(x), Large(y)) 
            | (Strict(x), Strict(y)) => Strict(x - y),
            (Large(x), Large(y)) => Large(x - y)
        }
    }
}

impl<T : Add<Output = T>> AddAssign for Bound<T> {
    fn add_assign(&mut self, rhs: Self) {
        let x = std::mem::take(self);
        *self = x + rhs
    }
}

impl<T : Sub<Output = T>> SubAssign for Bound<T> {
    fn sub_assign(&mut self, rhs: Self) {
        let x = std::mem::take(self);
        *self = x - rhs
    }
}

impl<T : PartialOrd> Mul for Bound<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

impl<T: Add<Output = T>> AddAssign<T> for Bound<T> {
    fn add_assign(&mut self, rhs: T) {
        *self += Large(rhs)
    }
}

impl<T: Sub<Output = T>> SubAssign<T> for Bound<T> {
    fn sub_assign(&mut self, rhs: T) {
        *self -= Large(rhs)
    }
}

impl<T : PartialOrd> MulAssign for Bound<T> {
    fn mul_assign(&mut self, rhs: Self) {
        self.mut_intersection(rhs);
    }
}

impl<T: Add<Output = T>> Add<T> for Bound<T> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        self + Large(rhs)
    }
}

impl<T: Sub<Output = T>> Sub<T> for Bound<T> {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        self - Large(rhs)
    }
}

impl<T : PartialOrd + Mul<Output = T> + Zero> Mul<T> for Bound<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        match self {
            Strict(x) => Strict(x * rhs),
            Large(x) => Large(x * rhs),
            Infinite => if rhs.is_zero() {
                Large(T::zero())
            } else if rhs < T::zero() {
                MinusInfinite
            } else {
                Infinite
            },
            MinusInfinite => if rhs.is_zero() {
                Large(T::zero())
            } else if rhs < T::zero() {
                Infinite
            } else {
                MinusInfinite
            },
        }
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

impl<T> Bounded for Bound<T> {
    fn min_value() -> Self {
        Bound::MinusInfinite
    }

    fn max_value() -> Self {
        Bound::Infinite
    }
}

impl<T: Add<Output = T>> Delta<T> for Bound<T> {
    fn delta(&mut self, dx: T) {
        *self += dx
    }
}

impl<T : PartialOrd> PartialOrd for Bound<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Infinite,Infinite) 
            | (MinusInfinite,MinusInfinite) => Some(std::cmp::Ordering::Equal),
            (Infinite, _) 
            | (_, MinusInfinite) => Some(std::cmp::Ordering::Greater),
            (_, Infinite) 
            | (MinusInfinite, _) => Some(std::cmp::Ordering::Less),
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

