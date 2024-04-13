use std::{fmt::Display, ops::{AddAssign, Sub}};

use num_traits::Zero;

pub struct DisjointInterval<T> {
    data : Vec<(T,T)>
}

impl<T : Clone + Zero + AddAssign + PartialOrd + Sub<Output = T>> DisjointInterval<T> {

    pub fn new() -> Self {
        DisjointInterval {
            data: Vec::new()
        }
    }

    pub fn from(a : T, b : T) -> Self {
        DisjointInterval {
            data: vec![(a,b)]
        }
    }

    pub fn contains(&self, x : T) -> bool {
        for r in self.data.iter() {
            if x < r.0 {
                return false;
            } else if x <= r.1 {
                return true;
            }
        }
        false
    }

    pub fn get(&self, x : T) -> T {
        let mut remain = x;
        for r in self.data.iter() {
            let length = r.1.clone() - r.0.clone();
            if remain > length {
                remain = remain - length;
            } else {
                return r.0.clone() + remain;
            }
        }
        remain
    }

    pub fn len(&self) -> T {
        let mut length = T::zero();
        for r in self.data.iter() {
            length = length + r.1.clone() - r.0.clone();
        }
        length
    }

    pub fn convex_equiv(&self) -> Self {
        Self::from(T::zero(), self.len())
    }

    pub fn min(&self) -> T {
        self.data[0].0.clone()
    }

    pub fn max(&self) -> T {
        self.data.last().unwrap().1.clone()
    }

    pub fn n_interv(&self) -> usize {
        self.data.len()
    }

    pub fn add_interval(&mut self, a : T, b : T) {
        if a > b { return }
        let mut a_contained = false;
        let mut b_contained = false;
        let mut i : usize = 0;
        let mut j : usize = 0;
        for (r_i, r) in self.data.iter().enumerate() {
            if r_i != i && r_i != j {
                break;
            }
            let begin = r.0.clone();
            let end = r.1.clone();
            if a.clone() >= begin.clone() && a.clone() <= end.clone() {
                a_contained = true;
            } else if a > end {
                i += 1;
            }
            if b.clone() >= begin.clone() && b.clone() <= end.clone() {
                b_contained = true;
            } else if b > end {
                j += 1;
            }
        }
        match (j - i, a_contained, b_contained) {
            (0, true, true) => (),
            (0, false, false) => self.data.insert(i, (a,b)),
            (_, true, false) => self.data[i].1 = b,
            (_, false, true) => self.data[i].0 = a,
            (_, true, true) => self.data[i].1 = self.data[j].1.clone(),
            (_, false, false) => self.data[i] = (a,b),
        }
        if i != j {
            for _ in 0..(j-i) { 
                if (i + 1) < self.data.len() { self.data.remove(i + 1); }
            }
        }
    }

}

impl<T : ToString> Display for DisjointInterval<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = String::new();
        for (a,b) in self.data.iter() {
            let current = format!("({},{})", a.to_string(), b.to_string());
            res += &current;
        }
        write!(f, "{{{}}}", res)
    }
}