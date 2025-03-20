use core::fmt;
use std::{collections::HashSet, hash::{DefaultHasher, Hash, Hasher}, sync::{RwLock, Weak}};

use nalgebra::DVector;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::{computation::{convex::{Convex, Measurable}, virtual_memory::{EvaluationType, VirtualMemory}, DBM}, models::{action::Action, model_var::ModelVar, petri::PetriNet, time::ClockValue, Label, ModelState, Node, UNMAPPED_ID}, verification::Verifiable};

#[derive(Debug, Serialize, Deserialize)]
pub struct StateClass {

    pub discrete : VirtualMemory,
    pub dbm : DBM,
    pub to_dbm_index : Vec<usize>,
    pub from_dbm_index : Vec<usize>,
    pub index : usize,

    #[serde(skip)]
    pub predecessors : RwLock<Vec<(Weak<StateClass>, Action)>>,

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
            storages : Vec::new(),
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
            if !state.is_enabled(transi.get_clock()) {
                continue;
            }
            let dbm_index = from_dbm.len();
            to_dbm[i] = dbm_index;
            from_dbm.push(i);
            dbm.add(dbm_index, 0, transi.interval.1);
            dbm.add(0, dbm_index, -transi.interval.0);
        }
        StateClass {
            discrete,
            dbm,
            to_dbm_index : to_dbm,
            from_dbm_index : from_dbm,
            predecessors : Default::default(),
            index : 0,
        }
    }

    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    pub fn down(&self) -> Self {
        let mut closed = self.clone();
        closed.dbm = closed.dbm.down();
        closed
    }

    pub fn up(&self) -> Self {
        let mut closed = self.clone();
        closed.dbm = closed.dbm.up();
        closed
    }

}

impl Verifiable for StateClass {

    fn evaluate_var(&self, var : &ModelVar) -> EvaluationType {
        self.discrete.evaluate(var)
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
impl Eq for StateClass { }

impl fmt::Display for StateClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut transitions = String::from("");
        if self.from_dbm_index.len() > 1 {
            transitions = self.from_dbm_index[1..].iter().map(|i| i.to_string()).collect::<Vec<String>>().join(",");
        }
        let preds = self.predecessors.read().unwrap().iter().map(|(p, a)| {
            let p = Weak::upgrade(p);
            format!("(Class_{}:{})", p.unwrap().index, a.get_id())
        }).collect::<Vec<String>>();
        let preds = preds.join(",");
        write!(f, "Class_{}\n- Marking {}\n- Transitions\n  [{}]\n- Predecessors\n  [{}]\n\n- {}", self.index, self.discrete, transitions, preds, self.dbm)
    }
}

impl Node for StateClass {
    fn get_label(&self) -> Label {
        Label::from("Class_:".to_owned() + &self.index.to_string())
    }
}

impl Clone for StateClass {
    fn clone(&self) -> Self {
        StateClass {
            discrete : self.discrete.clone(),
            dbm : self.dbm.clone(),
            to_dbm_index : self.to_dbm_index.clone(),
            from_dbm_index : self.from_dbm_index.clone(),
            index : UNMAPPED_ID,
            predecessors : Default::default(),
        }
    }
}

impl Measurable for StateClass {
    fn len(&self) -> f64 {
        self.dbm.len()
    }
}
