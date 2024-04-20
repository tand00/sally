use nalgebra::DVector;

use crate::{computation::DBM, verification::Verifiable};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StateClass {
    pub discrete : DVector<i32>,
    pub constraints : DBM,
}

impl Verifiable for StateClass {

    fn evaluate_object(&self, id : usize) -> i32 {
        self.discrete[id]
    }

    fn is_deadlocked(&self) -> bool {
        self.constraints.is_empty()
    }

}