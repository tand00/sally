use std::rc::Rc;

use num_traits::Zero;

use crate::{models::{action::Action, run::RunStatus, time::ClockValue, Model, ModelState}, verification::VerificationBound};

pub struct RandomRunIterator<'a> {
    pub model : &'a dyn Model,
    pub initial_state : &'a ModelState,
    pub run_status : RunStatus,
    pub bound : VerificationBound,
    pub started : bool,
}

impl<'a> RandomRunIterator<'a> {

    pub fn generate(model : &'a dyn Model, initial : &'a ModelState, bound : VerificationBound) -> Self {
        RandomRunIterator {
            model,
            initial_state : initial,
            run_status : RunStatus {
                current_state : Rc::new(initial.clone()),
                steps : 0,
                time : ClockValue::zero(),
                maximal : false
            },
            bound,
            started : false
        }
    }

    pub fn reset(&mut self) {
        self.run_status = RunStatus {
            current_state : Rc::new(self.initial_state.clone()),
            steps : 0,
            time : ClockValue::zero(),
            maximal : false
        };
        self.started = false;
    }

}

impl<'a> Iterator for RandomRunIterator<'a> {

    type Item = (Rc<ModelState>, ClockValue, Option<Action>);

    fn next(&mut self) -> Option<Self::Item> {
        
        if !self.started { // Yield the initial state
            self.started = true;
            return Some((Rc::clone(&self.run_status.current_state), ClockValue::zero(), None));
        }

        if self.run_status.maximal {
            return None;
        }

        let state = self.run_status.current_state.as_ref().clone();
        let (next_state, delay, action) = self.model.random_next(state);

        if next_state.is_none() {
            self.run_status.maximal = true;
            return None;
        }

        self.run_status.current_state = Rc::new(next_state.unwrap());
        self.run_status.steps += match action { None => 0, Some(_) => 1 };
        self.run_status.time += delay;

        if self.run_status.current_state.deadlocked {
            self.run_status.maximal = true;
        }

        if !self.run_status.is_under(&self.bound) {
            return None;
        }

        Some((Rc::clone(&self.run_status.current_state), delay, action))
    }

}