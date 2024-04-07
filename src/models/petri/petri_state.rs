use super::PetriMarking;
use crate::computation::DeltaList;
use crate::computation::ActionSet;

#[derive(Clone)]
pub struct FiringFunction {
    timings: DeltaList<f64>
}
impl FiringFunction {

    pub fn next_actions(&self) -> Vec<usize> {
        self.timings.index_min()
    }

    pub fn min_time(&self) -> f64 {
        self.timings.min_value()
    }

    pub fn step(&mut self, dt : f64) {
        self.timings.delta(-dt)
    }

    pub fn step_to_next_action(&mut self) {
        self.step(self.timings.min_value())
    }

    pub fn timing(&self, action : usize) -> f64{
        self.timings.at(action)
    }

    pub fn erase(&mut self, action : usize) {
        self.timings.remove(action);
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