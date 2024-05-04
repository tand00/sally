use crate::{models::{Model, ModelState}, verification::VerificationBound};
use crate::log::*;

use super::RandomRunIterator;


#[derive(Debug, Clone)]
pub struct SMCMaxSeen {
    pub runs_needed : usize,
}

impl SMCMaxSeen {

    pub fn new(runs : usize) -> Self {
        SMCMaxSeen {
            runs_needed : runs,
        }
    }

    pub fn estimate_max(&self, model : &impl Model, initial : &ModelState, bound : VerificationBound) -> i32 {
        info("Estimating max tokens using SMC...");
        continue_info(format!("Runs to be executed : {}", self.runs_needed));
        pending("Starting...");
        let mut max_seen = 0;
        for _ in 0..self.runs_needed {
            let iterator = RandomRunIterator::generate(model, initial, bound);
            for (state, _, _) in iterator {
                let tokens = state.discrete.sum();
                if tokens > max_seen {
                    max_seen = tokens;
                }
            }
        }
        positive(format!("Estimation complete, max seen : {}", max_seen));
        max_seen
    }

}