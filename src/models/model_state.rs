use std::{any::Any, collections::{HashMap, HashSet}};

use nalgebra::DVector;
use serde::{Deserialize, Serialize};

use crate::{computation::virtual_memory::{EvaluationType, VirtualMemory}, verification::Verifiable};

use super::{model_clock::ModelClock, model_storage::ModelStorage, model_var::ModelVar, time::ClockValue};

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct ModelState {
    pub discrete : VirtualMemory,
    pub clocks : DVector<ClockValue>,
    pub storages : Vec<ModelStorage>,
    pub deadlocked : bool,
}

impl ModelState {

    pub fn new(discrete_size : usize, clocks : usize) -> Self {
        ModelState {
            discrete : VirtualMemory::from_size(discrete_size),
            clocks :  DVector::from_element(clocks, ClockValue::disabled()),
            storages : Vec::new(),
            deadlocked : false
        }
    }

    pub fn step(&mut self, delta : ClockValue) {
        self.clocks.add_scalar_mut(delta)
    }

    pub fn step_with_rates(&mut self, rates : DVector<ClockValue>, delta : ClockValue) {
        self.clocks += rates * delta
    }

    pub fn enable_clock(&mut self, clock : &ModelClock, value : ClockValue) {
        self.clocks[clock.get_index()] = value
    }

    pub fn disable_clock(&mut self, clock : &ModelClock) {
        self.clocks[clock.get_index()] = ClockValue::disabled()
    }

    pub fn is_enabled(&self, clock : &ModelClock) -> bool {
        !self.clocks[clock.get_index()].is_disabled()
    }

    pub fn set_clock(&mut self, clock : &ModelClock, value : ClockValue) {
        self.clocks[clock.get_index()] = value
    }

    pub fn step_clock(&mut self, clock : &ModelClock, delta : ClockValue) {
        self.clocks[clock.get_index()] += delta;
    }

    pub fn step_clocks<'a>(&mut self, clocks : impl Iterator<Item = &'a ModelClock>, delta : ClockValue) {
        for clock in clocks {
            self.step_clock(clock, delta)
        }
    }

    pub fn get_clock_value(&self, clock : &ModelClock) -> ClockValue {
        self.clocks[clock.get_index()]
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

    pub fn marking_sum<'a>(&self, vars : impl Iterator<Item = &'a ModelVar>) -> EvaluationType {
        vars.map(|x| x.evaluate(self) ).sum()
    }

    pub fn argmax<'a>(&self, vars : impl Iterator<Item = &'a ModelVar>) -> usize {
        let mut max_i = 0;
        let mut max_value = EvaluationType::MIN;
        for (i, v) in vars.enumerate() {
            let value = self.evaluate_var(v);
            if value > max_value {
                max_i = i;
                max_value = value;
            }
        }
        max_i
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

    pub fn storage(&self, index : &usize) -> &ModelStorage {
        &self.storages[*index]
    }

    pub fn mut_storage(&mut self, index : &usize) -> &mut ModelStorage {
        &mut self.storages[*index]
    }

}

impl Verifiable for ModelState {

    fn evaluate_var(&self, var : &ModelVar) -> EvaluationType {
        self.discrete.evaluate(var)
    }

    fn evaluate_clock(&self, clock : &ModelClock) -> f64 {
        self.get_clock_value(clock).float()
    }

    fn is_deadlocked(&self) -> bool {
        self.deadlocked
    }

}

