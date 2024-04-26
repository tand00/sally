use std::collections::{HashMap, HashSet};

use nalgebra::DVector;

use crate::verification::Verifiable;

use super::{time::ClockValue, Label, Model};

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash)]
pub struct ModelState {
    pub discrete : DVector<i32>,
    pub clocks : DVector<ClockValue>,
    pub deadlocked : bool
}

impl ModelState {

    pub fn new(discrete_vars : usize, clocks : usize) -> Self {
        ModelState {
            discrete : DVector::zeros(discrete_vars),
            clocks :  DVector::from_element(clocks, ClockValue::disabled()),
            deadlocked : false
        }
    }

    pub fn from(model : &impl Model) -> Self {
        Self::new(model.n_vars(), model.n_clocks())
    }

    pub fn step(&mut self, delta : ClockValue) {
        self.clocks.add_scalar_mut(delta)
    }

    pub fn step_with_rates(&mut self, rates : DVector<ClockValue>, delta : ClockValue) {
        self.clocks += rates * delta
    }

    pub fn enable_clock(&mut self, clock : usize, value : ClockValue) {
        self.clocks[clock] = value
    }

    pub fn disable_clock(&mut self, clock : usize) {
        self.clocks[clock] = ClockValue::disabled()
    }

    pub fn is_enabled(&self, clock : usize) -> bool {
        !self.clocks[clock].is_disabled()
    }

    pub fn set_clock(&mut self, clock : usize, value : ClockValue) {
        self.clocks[clock] = value
    }

    pub fn set_marking(&mut self, id : usize, value : i32) {
        self.discrete[id] = value
    }

    pub fn get_marking(&self, id : usize) -> i32 {
        self.discrete[id]
    }

    pub fn is_marked(&self, id : usize) -> bool {
        self.discrete[id] > 0
    }

    pub fn enabled_clocks(&self) -> HashSet<usize> {
        let all_clocks = self.clocks.data.as_vec().to_vec();
        all_clocks.iter().enumerate().filter_map(|(i,c)| {
            if c.is_disabled() { None }
            else { Some(i) }
        }).collect()
    }

    pub fn covers(&self, marking : DVector<i32>) -> bool {
        self.discrete >= marking
    }

    pub fn tokens(&self, var : usize) -> i32 {
        self.discrete[var]
    }

    pub fn mark(&mut self, var : usize, tokens : i32) {
        self.discrete[var] += tokens
    }

    pub fn unmark(&mut self, var : usize, tokens : i32) {
        self.discrete[var] -= tokens
    }

}

impl Verifiable for ModelState {

    fn evaluate_object(&self, id : usize) -> i32 {
        self.discrete[id]
    }

    fn evaluate_clock(&self, id : usize) -> f64 {
        self.clocks[id].0
    }

    fn is_deadlocked(&self) -> bool {
        self.deadlocked
    }

}