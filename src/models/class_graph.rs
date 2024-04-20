mod state_class;
pub use state_class::StateClass;

use std::collections::HashSet;

use num_traits::Zero;

use super::time::ClockValue;
use super::{lbl, Edge, Model, ModelMeta, ModelState, CONTROLLABLE, SYMBOLIC, TIMED};
use super::petri::PetriNet;

pub struct ClassGraph {
    pub classes: Vec<StateClass>,
    pub edges: Vec<Edge<i32>>
}

impl ClassGraph {

    pub fn from(p_net : &PetriNet, initial_state : &ModelState) -> Self {
        let mut cg = ClassGraph {
            classes: Vec::new(),
            edges: Vec::new()
        };
        cg
    }

    pub fn successor(petri : &PetriNet, class : &StateClass, action : usize) -> StateClass {
        
        class.clone()
    }

}

impl Model for ClassGraph {

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name : lbl("ClassGraph"),
            description : String::from("Petri net Class graph, each node is associated with a DBM and is an aggregate of possible Petri states"),
            characteristics : TIMED | CONTROLLABLE | SYMBOLIC,
        }
    }

    fn next(&self, state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>) {
        (None, HashSet::new())
    }

    fn actions_available(&self, state : &ModelState) -> HashSet<usize> {
        HashSet::new()
    }

    fn available_delay(&self, state : &ModelState) -> ClockValue {
        ClockValue::zero()
    }

    fn n_vars(&self) -> usize {
        if self.classes.is_empty() {
            return 0;
        }
        self.classes.first().unwrap().discrete.nrows() + 1 // Additionnal discrete var to remember current class
    }

    fn n_clocks(&self) -> usize {
        if self.classes.is_empty() {
            return 0;
        }
        self.classes.first().unwrap().constraints.vars_count()
    }

}