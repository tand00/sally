use std::cmp::{max, min};

use nalgebra::DMatrix;
use rand::{thread_rng, Rng};

use super::genetic::Genetizable;

#[derive(Debug, Clone, Copy)]
pub enum Filter2D {
    IsAny,
    IsColor(usize),
    IsSymbol(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum Transformation2D {
    Identity,
    MirroredH,
    MirroredV,
    MirroredHV,
    Grow(usize),
    Rotate(usize),
}

pub struct Symbol;
pub struct Color(pub usize);

#[derive(Debug, Clone, Copy)]
pub enum SymbolID {
    Index(usize),
    FilterVar,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorID {
    Index(usize),
    FilterColor
}

pub struct LogicContext2D {
    pub symbols : Vec<Symbol>,
    pub colors : Vec<Color>
}

#[derive(Debug, Clone, Copy)]
pub enum Output2D {
    NoOutput,
    PutColor(ColorID),
    PutSymbol(Transformation2D, SymbolID),
}

pub struct Grid2D {
    matrix : DMatrix<Color>
}

#[derive(Debug, Clone, Copy)]
pub enum Movement2D {
    N, NE, E, SE, S, SW, W, NW
}

pub type StateID = usize;
pub const HALT : StateID = usize::MAX;

pub type Agent2DCase = (Filter2D, Output2D, Movement2D, StateID);
pub type Agent2DState = Vec<Agent2DCase>;

// Agent based on a Turing machine, adapted to move in 2D with filters instead of equalities,
// and feeding from a global context describing the environment
pub struct Agent2D {
    pub states : Vec<Agent2DState>,
}

impl Genetizable for Agent2D {

    fn cross(&self, other : &Self) -> Self {
        let mut rng = thread_rng();
        let i1 = rng.gen_range(0..self.states.len());
        let mut i2 = rng.gen_range(0..self.states.len());
        while i1 == i2 { i2 = rng.gen_range(0..self.states.len()); }
        let (i1, i2) = (min(i1,i2), max(i1,i2));
        let mut states : Vec<Agent2DState> = Vec::with_capacity(self.states.len());
        states[..i1].clone_from_slice(&self.states[..i1]);
        states[i1..i2].clone_from_slice(&other.states[i1..i2]);
        states[i2..].clone_from_slice(&self.states[i2..]);
        Agent2D { states }
    }

    fn mutate(&mut self) {
        let mut rng = thread_rng();
        let i = rng.gen_range(0..self.states.len());
        let state = &mut self.states[i];
    }
    
}