use super::PetriMarking;
use crate::computation::DeltaList;
use crate::computation::ActionSet;
use crate::verification::Verifiable;

#[derive(Clone)]
pub struct FiringFunction {
    timings: DeltaList<f64>,
    pub increasing : bool
}
impl FiringFunction {

    pub fn new() -> Self {
        FiringFunction {
            timings : DeltaList::new(0.0),
            increasing : false
        }
    }

    pub fn next_actions(&self) -> Vec<usize> {
        self.timings.index_min()
    }

    pub fn min_time(&self) -> f64 {
        self.timings.min_value()
    }

    pub fn step(&mut self, dt : f64) {
        let mult = if self.increasing { 1.0 } else { -1.0 };
        self.timings.delta(mult * dt)
    }

    pub fn step_to_next_action(&mut self) {
        self.step(self.timings.min_value())
    }

    pub fn timing(&self, action : usize) -> f64{
        self.timings.at(action)
    }

    pub fn set_timing(&mut self, action : usize, timing : f64) {
        self.timings.push(action, timing);
    }

    pub fn erase(&mut self, action : usize) {
        self.timings.remove(action);
    }

    pub fn merge(&mut self, other : FiringFunction) {
        self.timings.merge(other.timings);
    }

}

#[derive(Clone)]
pub struct PetriState {
    pub marking: PetriMarking,
    pub firing_function: FiringFunction,
    pub actions: ActionSet
}

impl PetriState {

    pub fn new_actions(&mut self, new_set : ActionSet) {
        let disabled = (&self.actions) & (&!&new_set);
        for a in disabled.get_actions() {
            self.firing_function.erase(a);
        }
        self.actions = new_set;
    }

}

impl Verifiable for PetriState {
    
    fn evaluate_object(&self, id : usize) -> i32 {
        self.marking.tokens(id)
    }

    fn is_deadlocked(&self) -> bool {
        (self.firing_function.min_time() == 0.0) && (self.actions.is_empty())
    }

}