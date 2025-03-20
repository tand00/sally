use core::fmt;
use std::{
    cmp::{max, min}, mem, ops::{Index, IndexMut}
};

use nalgebra::{DMatrix, DVector};
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::models::time::{Bound, ClockValue, Interval};

use super::convex::{ContinuousSet, Convex, Delta, Disjoint, Measurable};

pub type IntBound = Bound<i32>;
pub type IntInterval = Interval<i32>;
pub type DatesVector = DVector<ClockValue>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DBM {
    constraints: DMatrix<IntBound>,
}

// We add an imaginary variable, always equal to zero, at the beginning of the matrix. That way, we can encode rectangular constraints
// Algorithms inspired by Bengtsson et Yi, « Timed Automata » https://doi.org/10.1007/978-3-540-27755-2_3.
impl DBM {

    pub fn new(vars: usize) -> Self {
        DBM {
            constraints: DMatrix::from_fn(vars + 1, vars + 1, |i, j| {
                if i == j {
                    IntBound::zero()
                } else {
                    IntBound::Infinite
                }
            }),
        }
    }

    pub fn from(constraints: DMatrix<IntBound>) -> Self {
        if !constraints.is_square() {
            panic!("Constraints matrix not square, can't construct DBM !");
        }
        let mut res = DBM { constraints };
        res.make_canonical();
        res
    }

    pub fn empty(vars: usize) -> Self {
        DBM {
            constraints: DMatrix::from_element(vars + 1, vars + 1, IntBound::MinusInfinite),
        }
    }

    pub fn at(&self, i: usize, j: usize) -> IntBound {
        self[(i, j)]
    }

    pub fn rectangulars(&self, i: usize) -> IntInterval {
        IntInterval::new(-self.constraints[(0, i)], self.constraints[(i, 0)])
    }

    pub fn diagonals(&self, i: usize, j: usize) -> IntInterval {
        IntInterval::new(-self.constraints[(j, i)], self.constraints[(i, j)])
    }

    pub fn vars_count(&self) -> usize {
        self.constraints.nrows() - 1
    }

    pub fn get_canonical(&self) -> Self {
        let mut canonical = self.clone();
        canonical.make_canonical();
        canonical
    }

    pub fn set_upper_bound(&mut self, var_i: usize, bound: IntBound) {
        self.add(var_i, 0, bound)
    }

    pub fn set_lower_bound(&mut self, var_i: usize, bound: IntBound) {
        self.add(0, var_i, -bound)
    }

    pub fn free_clock(&mut self, var_i: usize) {
        for i in 0..(self.vars_count() + 1) {
            if i == var_i {
                continue;
            }
            self.constraints[(i, var_i)] = self.constraints[(i, 0)];
            self.constraints[(var_i, i)] = IntBound::Infinite;
        }
    }

    pub fn make_empty(&mut self) {
        //*self = Self::empty(self.vars_count());
        self[(0,0)] = IntBound::MinusInfinite;
    }

    pub fn add(&mut self, var_i: usize, var_j: usize, constraint: IntBound) {
        if self.constraints[(var_j, var_i)] + constraint < IntBound::zero() {
            self.make_empty();
        }
        let current = &mut self.constraints[(var_i, var_j)];
        if constraint < *current {
            *current = constraint;
            let n_rows = self.constraints.nrows();
            for i in 0..n_rows {
                for j in 0..n_rows {
                    self.constraints[(i, j)] = min(
                        self.constraints[(i, j)],
                        self.constraints[(i, var_i)] + self.constraints[(var_i, j)],
                    );
                    self.constraints[(i, j)] = min(
                        self.constraints[(i, j)],
                        self.constraints[(i, var_j)] + self.constraints[(var_j, j)],
                    );
                }
            }
        }
    }

    pub fn add_sup(&mut self, var_i : usize, constraint : IntBound) {
        self.add(var_i, 0, constraint);
    }
    
    pub fn add_inf(&mut self, var_i : usize, constraint : IntBound) {
        self.add(0, var_i, -constraint);
    }

    pub fn remove_var(&mut self, var_i: usize) {
        //self.free_clock(var_i);
        let mut constraints = mem::replace(
            &mut self.constraints,
            DMatrix::from_vec(0, 0, vec![])
        );
        constraints = constraints.remove_column(var_i).remove_row(var_i);
        self.constraints = constraints;
    }

    pub fn existential_projection(&self, vars : &Vec<usize>) -> DBM {
        let n = vars.len() + 1;
        let mut rows = DMatrix::zeros(n, self.constraints.ncols());
        let mut constraints = DMatrix::zeros(n,n);
        for (i, var) in vars.iter().enumerate() {
            rows.row_mut(i).copy_from(&self.constraints.row(*var));
        }
        for (j, var) in vars.iter().enumerate() {
            constraints.column_mut(j).copy_from(&rows.column(*var));
        }
        DBM { constraints }
    }

    pub fn universal_projection(&self, other : &DBM, vars : &Vec<usize>) -> ContinuousSet<DatesVector,DBM> {
        let interm = ContinuousSet::from(other.clone()).complement();
        let mut interm = interm.intersection(self.clone());

        interm.apply_fn(|x : &mut DBM| {
            *x = x.existential_projection(vars)
        });
        let interm = interm.complement();

        ContinuousSet::from(self.existential_projection(vars)).intersection(interm)
    }

    pub fn extend(self, nvars : usize) -> DBM {
        let before = self.constraints.nrows();
        let n = before + nvars;
        let mut constraints = self.constraints.resize(n, n, Bound::Infinite);
        for i in 0..nvars {
            constraints[(before + i, before + i)] = Bound::Large(0);
        }
        DBM { constraints }
    }

    pub fn extend_to(self, nvars : usize) -> DBM {
        let vars = self.vars_count();
        if vars >= nvars {
            return self
        }
        let new_vars = nvars - vars;
        self.extend(new_vars)
    }

    pub fn extend_mut(&mut self, nvars : usize) {
        let before = self.constraints.nrows();
        let n = before + nvars;
        self.constraints.resize_mut(n, n, Bound::Infinite);
        for i in 0..nvars {
            self.constraints[(before + i, before + i)] = Bound::Large(0);
        }
    }

    pub fn extend_to_mut(&mut self, nvars : usize) {
        let vars = self.vars_count();
        if vars < nvars {
            let new_vars = nvars - vars;
            self.extend_mut(new_vars);
        }
    }

    pub fn make_canonical(&mut self) {
        let n_rows = self.constraints.nrows();
        for k in 0..n_rows {
            for i in 0..n_rows {
                for j in 0..n_rows {
                    self.constraints[(i, j)] = min(
                        self.constraints[(i, j)],
                        self.constraints[(i, k)] + self.constraints[(k, j)],
                    );
                    if i == j && self.constraints[(i, j)] < IntBound::zero() {
                        self.make_empty();
                        return;
                    }
                }
            }
        }
    }

    pub fn down(&self) -> DBM {
        // Assuming DBM is canonical
        let mut res = self.clone();
        if res.is_empty() { return res; }
        let max_delta = self.constraints.column(0).iter().min().unwrap().clone();
        for i in 1..(self.vars_count() + 1) {
            res.constraints[(0, i)] = min(IntBound::zero(), self.constraints[(0, i)] + max_delta);
        }
        res
    }

    pub fn up(&self) -> DBM {
        let mut res = self.clone();
        if res.is_empty() { return res; }
        for i in 1..(self.vars_count() + 1) {
            res.constraints[(i, 0)] = IntBound::Infinite;
        }
        res
    }

}

impl Index<(usize, usize)> for DBM {
    type Output = IntBound;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.constraints[index] // Index + 1 because virtual var at 0
    }
}

// Prefer using 'add', this will overwrite the current constraint and potentially not preserve the canonical structure
impl IndexMut<(usize, usize)> for DBM {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.constraints[index]
    }
}

impl fmt::Display for DBM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DBM{}", self.constraints)
    }
}

impl Convex<DatesVector> for DBM {

    fn contains(&self, elem: &DatesVector) -> bool {
        if self.vars_count() == 0 {
            return true;
        }
        if elem.len() != self.vars_count() || self.is_empty() {
            return false;
        }
        let vars = elem.len();
        for i in 0..vars {
            if !self.rectangulars(i + 1).contains(&elem[i]) {
                return false;
            }
            for j in (i + 1)..vars {
                let diag = elem[i] - elem[j];
                if !self.diagonals(i + 1, j + 1).contains(&diag) {
                    return false;
                }
            }
        }
        true
    }

    fn intersection(mut self, mut other: Self) -> Self {
        if other.is_empty() || self.is_empty() {
            return Self::empty(self.vars_count());
        } else if other.vars_count() != self.vars_count() {
            let nvars = max(self.vars_count(), other.vars_count());
            self.extend_to_mut(nvars);
            other.extend_to_mut(nvars);
        }
        self.mut_intersect(other);
        self
    }

    fn full() -> Self {
        Self::new(0)
    }

    fn is_empty(&self) -> bool {
        self.constraints[(0, 0)] < IntBound::zero()
    }

    fn union(mut self, mut other: Self) -> Disjoint<DatesVector, Self> {
        if other.is_empty() {
            return self.into();
        } else if self.is_empty() {
            return other.into();
        } else if other.vars_count() != self.vars_count() {
            let nvars = max(self.vars_count(), other.vars_count());
            self.extend_to_mut(nvars);
            other.extend_to_mut(nvars);
        }
        if self == other {
            return self.into()
        }
        Disjoint::from((self, other))
    }

    fn complement(self) -> Disjoint<DatesVector, Self> {
        let n = self.constraints.nrows();
        let base = DBM::new(n - 1);
        if self.is_empty() {
            return base.into();
        }
        let mut res = Vec::new();
        for i in 0..n {
            for j in 0..n {
                let constraint = self[(i,j)];
                if (i == j) || (constraint == IntBound::Infinite) {
                    continue;
                }
                let mut negated = base.clone();
                negated[(j,i)] = -!constraint;
                res.push(negated);
            }
        }
        Disjoint::from(res)
    }

    fn intersects(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        let vars = min(self.vars_count(), other.vars_count());
        for i in 0..vars {
            let self_rect = self.rectangulars(i + 1);
            let other_rect = other.rectangulars(i + 1);
            if !self_rect.intersects(&other_rect) {
                return false;
            }
            for j in (i + 1)..vars {
                let self_diag = self.diagonals(i + 1, j + 1);
                let other_diag = other.diagonals(i + 1, j + 1);
                if !self_diag.intersects(&other_diag) {
                    return false;
                }
            }
        }
        true
    }

    fn mut_intersect(&mut self, other: Self) {
        self.constraints.component_mul_assign(&other.constraints);
        self.make_canonical();
    }

    fn covers(&self, other: &Self) -> bool {
        if self.vars_count() != other.vars_count() {
            return false;
        }
        self.constraints >= other.constraints
    }

    // Might cause bad performances, but ensure no duplicates... I don't know
    fn fuse(set: &mut Vec<Self>, elem: Self) {
        for x in set.iter() {
            if x.covers(&elem) {
                return;
            }
        }
        set.push(elem)
    }

}

impl Delta<IntBound> for DBM {
    fn delta(&mut self, dx: IntBound) {
        if self.is_empty() { return; }
        for i in 1..(self.vars_count() + 1) {
            self.constraints[(i, 0)] += dx;
            self.constraints[(0, i)] -= dx;
        }
    }
}

/// Experimental measure, volume of the englobing square
impl Measurable for DBM {
    fn len(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        let mut s = 1.0f64;
        for i in 0..self.vars_count() {
            let rect = self.rectangulars(i + 1);
            s *= rect.len();
            if s.is_zero() {
                return 0.0;
            }
        }
        s
    }
}
