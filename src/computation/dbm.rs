use std::{cmp::min, iter::zip, ops::{Index, IndexMut, Not}};

use crate::models::time::{TimeBound, TimeBound::{Strict, Large, Infinite, MinusInfinite}};

#[derive(Clone, PartialEq, Eq)]
pub struct DBM {
    vars : usize,
    constraints : Vec<TimeBound>,
}

// We add an imaginary variable, always equal to zero, at the end of the matrix. That way, we can encode rectangular constraints
impl DBM {

    pub fn new(vars : usize) -> Self {
        let mut matrix = DBM {
            vars,
            constraints: Vec::new()
        };
        matrix.constraints.resize((vars + 1) * (vars + 1), Infinite);
        for i in 0..(vars + 1) {
            matrix[(i,i)] = Large(0);
        }
        matrix
    }

    pub fn at(&self, i : usize, j : usize) -> TimeBound {
        self[(i,j)]
    }

    pub fn rectangulars(&self, i : usize) -> (TimeBound, TimeBound) {
        let index_1 = i * (self.vars + 1) + 0;
        let index_2 = i;
        (-self.constraints[index_2], self.constraints[index_1])
    }

    pub fn intersection(&self, other : &DBM) -> Self {
        let mins = 
            zip(&self.constraints, &other.constraints)
            .map(|(a,b)| a.intersection(*b)).collect();
        DBM {
            vars: min(self.vars, other.vars),
            constraints: mins
        }
    }

    pub fn contains(&self, other : &DBM) -> bool {
        if self.vars != other.vars {
            return false;
        }
        let any_sup = 
            zip(&self.constraints, &other.constraints)
            .any(|(a,b)| (*b) > (*a)); // Any because more optimized than All (might be detail)
        !any_sup
    }

    pub fn canonical(self) -> Self {
        self
    }

}

impl Index<(usize, usize)> for DBM {
    type Output = TimeBound;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let index = (index.0 * (self.vars + 1)) + index.1;
        &self.constraints[index]
    }
}

impl IndexMut<(usize, usize)> for DBM {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let index = (index.0 * (self.vars + 1)) + index.1;
        &mut self.constraints[index]
    }
}