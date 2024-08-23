use crate::{log, solution::SolverResult, verification::VerificationStatus};

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

    pub fn fixed_runs(runs : usize, confidence : f64) -> Self {
        ProbabilityEstimation {
            confidence, 
            interval_width : (4.0 * (2.0 / (1.0 - confidence)).ln() / (runs as f64)).sqrt(),
            runs_needed : runs,
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

    fn prepare(&self) {
        log::continue_info("Type : Probability estimation");
        log::continue_info(format!("Confidence : {}%", self.confidence * 100.0));
        log::continue_info(format!("Interval width : {}", self.interval_width));
        log::continue_info(format!("Need to execute [{}] runs", self.runs_needed));
    }

    fn finish(&self) {
        log::continue_info(format!("Valid runs : [{}]", self.valid_runs));
    }

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
