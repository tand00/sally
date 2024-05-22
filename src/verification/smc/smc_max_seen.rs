use crate::{models::{model_context::ModelContext, model_var::ModelVar, Model, ModelState}, verification::VerificationBound};
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

    pub fn estimate_max(&self, ctx : &ModelContext, model : &impl Model, initial : &ModelState, bound : VerificationBound) -> i32 {
        info("Estimating max tokens using SMC...");
        continue_info(format!("Runs to be executed : {}", self.runs_needed));
        pending("Starting...");
        let mut max_seen = 0;
        let vars = ctx.get_vars();
        let vars_refs : Vec<&ModelVar> = vars.iter().collect();
        for _ in 0..self.runs_needed {
            let iterator = RandomRunIterator::generate(model, initial, bound);
            for (state, _, _) in iterator {
                let tokens = state.marking_sum(vars_refs);
                if tokens > max_seen {
                    max_seen = tokens;
                }
            }
        }
        positive(format!("Estimation complete, max seen : {}", max_seen));
        max_seen
    }

}