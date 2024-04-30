use core::fmt;
use std::{cell::RefCell, collections::HashSet, hash::{DefaultHasher, Hash, Hasher}, rc::Weak};

use nalgebra::DVector;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::{computation::DBM, models::{petri::PetriNet, time::ClockValue, Label, ModelState, Node}, verification::Verifiable};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateClass {
    pub discrete : DVector<i32>,
    pub dbm : DBM,
    pub to_dbm_index : Vec<usize>,
    pub from_dbm_index : Vec<usize>,
    pub index : usize,

    #[serde(skip)]
    pub predecessors : Vec<(Weak<RefCell<StateClass>>, usize)>,
    
}

impl StateClass {
    
    pub fn generate_image_state(&self) -> ModelState {
        let deadlocked = self.is_deadlocked();
        let clocks : Vec<ClockValue> = self.to_dbm_index.iter().enumerate().map(|(_, i)| {
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
        let mut to_dbm = vec![0; petri.transitions.len()];
        let mut from_dbm = vec![0];
        for (i, transi) in petri.transitions.iter().enumerate() {
            if !state.is_enabled(i) {
                continue;
            }
            let dbm_index = from_dbm.len();
            to_dbm[i] = dbm_index;
            from_dbm.push(i);
            dbm.add(dbm_index, 0, transi.borrow().interval.1);
            dbm.add(0, dbm_index, -transi.borrow().interval.0);
        }
        StateClass {
            discrete,
            dbm,
            to_dbm_index : to_dbm,
            from_dbm_index : from_dbm,
            predecessors : Vec::new(),
            index : 0,
        }
    }

    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    pub fn as_verifiable(&self) -> &impl Verifiable {
        self
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

impl Hash for StateClass {

    fn hash<H: Hasher>(&self, state: &mut H) { // Every other field is rendundant
        self.discrete.hash(state);
        self.dbm.hash(state);
    }

}

impl PartialEq for StateClass {
    fn eq(&self, other: &Self) -> bool {
        (self.discrete == other.discrete) && (self.dbm == other.dbm)
    }
}

impl fmt::Display for StateClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut transitions = String::from("");
        if self.from_dbm_index.len() > 1 {
            transitions = self.from_dbm_index[1..].iter().map(|i| i.to_string()).collect::<Vec<String>>().join(",");
        }
        write!(f, "Class_{}\n- Marking {}- Transitions\n  [{}]\n\n- {}", self.index, self.discrete, transitions, self.dbm)
    }
}

impl Node for StateClass {
    fn get_label(&self) -> Label {
        Label::from("Class_:".to_owned() + &self.index.to_string())
    }
}