use std::{collections::HashSet, hash::{DefaultHasher, Hash, Hasher}};

use nalgebra::DVector;
use num_traits::Zero;

use crate::{computation::DBM, models::{petri::PetriNet, time::ClockValue, Model, ModelState}, verification::Verifiable};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StateClass {
    pub discrete : DVector<i32>,
    pub dbm : DBM,
    pub to_dbm_index : Vec<usize>,
    pub from_dbm_index : Vec<usize>,
    pub predecessors : Vec<(usize,usize)>, // Pred index, Action
}

impl StateClass {
    
    pub fn generate_image_state(&self) -> ModelState {
        let deadlocked = self.is_deadlocked();
        let clocks : Vec<ClockValue> = self.to_dbm_index.iter().enumerate().map(|(t,i)| {
            if *i == 0 {
                ClockValue::disabled()
            } else {
                ClockValue::zero()
            }
        }).collect();
        ModelState {
            discrete : self.discrete.clone(),
            clocks : DVector::from(clocks),
            deadlocked
        }
    }

    pub fn enabled_clocks(&self) -> HashSet<usize> {
        let mut res = HashSet::new();
        if self.from_dbm_index.len() <= 1 {
            return res;
        }
        for clock in self.from_dbm_index[1..].iter() {
            res.insert(*clock);
        }
        res
    }

    pub fn compute_class(petri : &PetriNet, state : &ModelState) -> Self {
        let discrete = state.discrete.clone();
        let enabled_clocks = state.enabled_clocks().len();
        let mut dbm = DBM::new(enabled_clocks);
        let mut to_dbm = Vec::new();
        let mut from_dbm = vec![0];
        for (i, transi) in petri.transitions.iter().enumerate() {
            if !state.is_enabled(i) {
                continue;
            }
            let dbm_index = from_dbm.len();
            to_dbm[i] = dbm_index;
            from_dbm.push(i);
            dbm.add(dbm_index, 0, transi.interval.1);
            dbm.add(0, dbm_index, transi.interval.0);
        }
        StateClass {
            discrete,
            dbm,
            to_dbm_index : to_dbm,
            from_dbm_index : from_dbm,
            predecessors : Vec::new()
        }
    }

    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

}

impl Verifiable for StateClass {

    fn evaluate_object(&self, id : usize) -> i32 {
        self.discrete[id]
    }

    fn is_deadlocked(&self) -> bool {
        self.dbm.vars_count() == 0 || self.dbm.is_empty() // DBM should not be empty in a state class !
    }

}

impl From<StateClass> for ModelState {
    fn from(value: StateClass) -> Self {
        value.generate_image_state()
    }
}