use std::{collections::HashSet, rc::Rc};

use nalgebra::DVector;
use serde::{Deserialize, Serialize};

use crate::{computation::virtual_memory::{EvaluationType, VariableDefiner, VirtualMemory}, verification::Verifiable};

use super::{model_var::ModelVar, time::ClockValue, Model};

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct ModelState {
    pub discrete : VirtualMemory,
    pub clocks : DVector<ClockValue>,
    pub deadlocked : bool
}

impl ModelState {

    pub fn new(discrete_size : usize, clocks : usize) -> Self {
        ModelState {
            discrete : VirtualMemory::from_size(discrete_size),
            clocks :  DVector::from_element(clocks, ClockValue::disabled()),
            deadlocked : false
        }
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

    pub fn set_marking(&mut self, var : &ModelVar, value : EvaluationType) {
        self.discrete.set(var, value);
    }

    pub fn get_marking(&self, var : &ModelVar) -> EvaluationType {
        self.discrete.evaluate(var)
    }

    pub fn is_marked(&self, var : &ModelVar) -> bool {
        self.get_marking(var) > 0
    }

    pub fn enabled_clocks(&self) -> HashSet<usize> {
        let all_clocks = self.clocks.data.as_vec().to_vec();
        all_clocks.iter().enumerate().filter_map(|(i,c)| {
            if c.is_disabled() { None }
            else { Some(i) }
        }).collect()
    }

    pub fn tokens(&self, var : &ModelVar) -> EvaluationType {
        self.get_marking(var)
    }

    pub fn marking_sum(&self, vars : Vec<&ModelVar>) -> EvaluationType {
        vars.iter().map(|x| x.evaluate(self) ).sum()
    }

    pub fn mark(&mut self, var : &ModelVar, tokens : EvaluationType) {
        self.discrete.set(var, self.get_marking(var) + tokens)
    }

    pub fn unmark(&mut self, var : &ModelVar, tokens : EvaluationType) {
        self.discrete.set(var, self.get_marking(var) - tokens)
    }

    pub fn create_clocks(&mut self, clocks : usize) {
        self.clocks = DVector::from_element(clocks, ClockValue::disabled())
    }

}

impl Verifiable for ModelState {

    fn evaluate_var(&self, var : &ModelVar) -> EvaluationType {
        self.discrete.evaluate(var)
    }

    fn evaluate_clock(&self, id : usize) -> f64 {
        self.clocks[id].0
    }

    fn is_deadlocked(&self) -> bool {
        self.deadlocked
    }

}

