use super::{Label, Model, State, Transition};

pub trait ObservationFunction {
    fn observe_state(&self, state : &Box<dyn State>) -> Label;
    fn observe_transition(&self, action : &Box<dyn Transition>) -> Label;
}