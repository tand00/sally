use core::fmt;
use std::io::{Read, Write};
use std::{cmp::min, ops::{Index, IndexMut}};

use nalgebra::DMatrix;
use num_traits::{Bounded, Zero};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::models::time::{ClockValue, TimeBound, TimeInterval};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DBM {
    vars : usize,
    constraints : DMatrix<TimeBound>
}

// We add an imaginary variable, always equal to zero, at the end of the matrix. That way, we can encode rectangular constraints
impl DBM {

    pub fn new(vars : usize) -> Self {
        DBM {
            vars,
            constraints: DMatrix::from_fn(vars + 1, vars + 1, |i,j| {
                if i == j { TimeBound::zero() }
                else { TimeBound::max_value() }
            })
        }
    }

    pub fn from(constraints : DMatrix<TimeBound>) -> Self {
        if !constraints.is_square() {
            panic!("Constraints matrix not square, can't construct DBM !");
        }
        let mut res = DBM {
            vars : constraints.nrows(),
            constraints,
        };
        res.make_canonical();
        res
    }

    pub fn empty(vars : usize) -> Self {
        DBM {
            vars,
            constraints: DMatrix::from_element(vars + 1, vars + 1 , TimeBound::MinusInfinite)
        }
    }

    pub fn at(&self, i : usize, j : usize) -> TimeBound {
        self[(i,j)]
    }

    pub fn rectangulars(&self, i : usize) -> TimeInterval {
        TimeInterval(
            -self.constraints[(0, i)], 
            self.constraints[(i, 0)]
        )
    }

    pub fn diagonals(&self, i : usize, j : usize) -> TimeInterval {
        TimeInterval(
            -self.constraints[(j, i)], 
            self.constraints[(i, j)]
        )
    }

    pub fn intersection(&self, other : &DBM) -> Self {
        DBM {
            vars: min(self.vars, other.vars),
            constraints: self.constraints.component_mul(&other.constraints)
        }
    }

    pub fn contains(&self, other : &DBM) -> bool {
        if self.vars != other.vars {
            return false;
        }
        self.constraints >= other.constraints
    }

    pub fn vars_count(&self) -> usize {
        self.vars
    }

    pub fn get_canonical(&self) -> Self {
        let mut canonical = self.clone();
        canonical.make_canonical();
        canonical
    }

    pub fn set_bound(&mut self, var_i : usize, bound : TimeBound) {
        self.add(var_i, 0, bound)
    }

    pub fn free_clock(&mut self, var_i : usize) {
        for i in 0..(self.vars + 1) {
            if i == var_i {
                continue;
            }
            self.constraints[(i, var_i)] = self.constraints[(i, 0)];
            self.constraints[(var_i, i)] = TimeBound::Infinite;
        }
    }

    pub fn add(&mut self, var_i : usize, var_j : usize, constraint : TimeBound) {
        let current = &mut self.constraints[(var_i, var_j)];
        if *current + constraint < TimeBound::zero() {
            *self = Self::empty(self.vars);
        } else if constraint < *current {
            *current = constraint;
            let n_rows = self.constraints.nrows();
            for i in 0..n_rows {
                for j in 0..n_rows {
                    self.constraints[(i,j)] = min(
                        self.constraints[(i,j)],
                        self.constraints[(i, var_i)] + self.constraints[(var_i, j)] 
                    );
                    self.constraints[(i,j)] = min(
                        self.constraints[(i,j)],
                        self.constraints[(i, var_j)] + self.constraints[(var_j, j)] 
                    );
                }
            }
        }
    }

    pub fn remove_var(&mut self, var_i : usize) {
        //self.free_clock(var_i);
        self.constraints = self.constraints.clone().remove_column(var_i).remove_column(var_i);
        
    }

    pub fn make_canonical(&mut self) {
        let n_rows = self.constraints.nrows();
        for k in 0..n_rows {
            for i in 0..n_rows {
                for j in 0..n_rows {
                    self.constraints[(i,j)] = min(
                        self.constraints[(i,j)],
                        self.constraints[(i,k)] + self.constraints[(k,j)] 
                    );
                    if i == j && self.constraints[(i,j)] < TimeBound::zero() {
                        *self = Self::empty(self.vars);
                        return;
                    }
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.constraints[(0,0)] < TimeBound::zero()
    }

    pub fn delta(&mut self, delta : TimeBound) {
        for i in 1..(self.vars + 1) {
            self.constraints[(i,0)] += delta;
            self.constraints[(0,i)] -= delta;
        }
    }

    pub fn closure(&self) -> DBM {
        let mut res = self.clone();
        let max_delta = self.constraints.column(0).iter().min().unwrap().clone();
        
        res
    }

}

impl Index<(usize, usize)> for DBM {
    type Output = TimeBound;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.constraints[index] // Index + 1 because virtual var at 0
    }
}

// Prefer using 'add', this will overwrite the current constraint and potentially not preserve the canonical structure
impl IndexMut<(usize,usize)> for DBM {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.constraints[index]
    }
}

impl fmt::Display for DBM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /*let mut lines = Vec::new();
        for i in 0..(self.vars + 1) {
            let mut line_i : Vec<u8> = Vec::new();
            for j in 0..(self.vars + 1) {
                write!(line_i, "\t{},", self[(i,j)]);
            }
            let line_i = String::from_utf8(line_i).unwrap();
            lines.push(line_i);
        }
        write!(f, "DBM(\n{}\n)", lines.join("\n"))*/
        write!(f, "DBM{}", self.constraints)
    }
}