use std::rc::Rc;

use crate::verification::{VerificationBound, Verifiable};

use super::{time::ClockValue, ModelState};

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
            TimeRunBound(t) => self.time < ClockValue(*t as f64),
            StepsRunBound(s) => self.steps < *s,
            VarRunBound(v, x) => self.current_state.evaluate_var(v) < *x,
            NoRunBound => true
        }
    }

}