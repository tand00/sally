use std::rc::Rc;

use crate::verification::{VerificationBound, Verifiable};

use super::{action::Action, time::ClockValue, ModelState};

use num_traits::Zero;
use VerificationBound::*;

#[derive(Debug, Clone, PartialEq)]
pub struct RunStatus {
    pub current_state : Rc<ModelState>,
    pub steps : usize,
    pub time : ClockValue,
    pub maximal : bool
}

impl RunStatus {

    pub fn is_under(&self, bound : &VerificationBound) -> bool {
        match bound {
            TimeRunBound(t) => self.time < ClockValue::from(*t as f64),
            StepsRunBound(s) => self.steps < *s,
            VarRunBound(v, x) => self.current_state.evaluate_var(v) < *x,
            NoRunBound => true
        }
    }

}

pub enum RunElement {
    State(Rc<ModelState>),
    Step(Action),
    Delay(ClockValue)
}

use RunElement::*;

pub struct Run {
    pub elements : Vec<RunElement>,
    pub steps : usize,
    pub time : ClockValue,
    pub maximal : bool
}

impl Run {

    pub fn new() -> Run {
        Run {
            elements : Vec::new(),
            steps : 0,
            time : ClockValue::zero(),
            maximal : false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn last_state(&self) -> Option<Rc<ModelState>> {
        for i in self.elements.iter().rev() {
            if let State(s) = i {
                return Some(Rc::clone(s))
            }
        }
        None
    }

    pub fn add(&mut self, elem : RunElement) {
        match &elem {
            Step(_) => self.steps += 1,
            Delay(d) => self.time += d.clone(),
            _ => ()
        }
        self.elements.push(elem);
    }

    pub fn trace(&self) -> Run {
        let mut res = Run::new();
        for elem in self.elements.iter() {
            match elem {
                Step(a) => if !a.is_epsilon() {
                    res.add(Step(a.clone()))
                },
                _ => ()
            }
        }
        res.maximal = self.maximal;
        res.time = self.time;
        res
    }

}