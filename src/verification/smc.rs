mod random_run_generator;
mod probability_estimation;
mod probability_float_comparison;

pub use random_run_generator::RandomRunIterator;
pub use probability_estimation::ProbabilityEstimation;
pub use probability_float_comparison::ProbabilityFloatComparison;

use crate::{models::{Model, ModelState}, solution::SolverResult, Query};

use super::{VerificationStatus, Verifiable};

pub trait SMCQueryVerification {

    fn must_do_another_run(&self) -> bool;
    fn handle_run_result(&mut self, result : VerificationStatus);
    fn get_result(&self) -> SolverResult;

    fn verify(&mut self, model : &impl Model, initial_state : &ModelState, query : &Query) -> SolverResult {
        let mut query = query.clone();
        while self.must_do_another_run() {
            let result = self.execute_run(model, initial_state, &mut query);
            self.handle_run_result(result);
        }
        self.get_result()
    }

    fn execute_run(&mut self, model : &impl Model, initial_state : &ModelState, query : &mut Query) -> VerificationStatus {
        let run_gen = RandomRunIterator::generate(model, initial_state, query.run_bound);
        for (state, _, _) in run_gen {
            query.verify_state(state.as_verifiable());
            if query.is_run_decided() {
                break;
            }
        }
        query.end_run();
        let result = query.run_status;
        query.reset_run();
        result
    }

}
