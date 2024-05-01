use crate::{solution::SolverResult, verification::VerificationStatus};

use super::SMCQueryVerification;

#[derive(Debug, Clone)]
pub struct ProbabilityEstimation {
    pub confidence : f64,
    pub interval_width : f64,
    pub runs_needed : usize,
    pub executed_runs : usize,
    pub valid_runs : usize,
}

impl ProbabilityEstimation {

    pub fn new(confidence : f64, interval_width : f64) -> Self {
        ProbabilityEstimation {
            confidence, interval_width,
            runs_needed : Self::chernoff_hoeffding_bound(confidence, interval_width),
            executed_runs : 0,
            valid_runs: 0
        }
    }

    fn chernoff_hoeffding_bound(confidence : f64, interval_width : f64) -> usize {
        let bound = 4.0 * (2.0 / (1.0 - confidence)).ln() / interval_width.powi(2);
        bound.ceil() as usize
    }

}

impl SMCQueryVerification for ProbabilityEstimation {

    fn must_do_another_run(&self) -> bool {
        self.executed_runs < self.runs_needed
    }

    fn handle_run_result(&mut self, result : VerificationStatus) {
        if result.good() {
            self.valid_runs += 1;
        }
        self.executed_runs += 1;
    }

    fn get_result(&self) -> SolverResult {
        SolverResult::FloatResult( (self.valid_runs as f64) / (self.executed_runs as f64) )
    }

}
