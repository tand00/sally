use crate::{solution::SolverResult, verification::VerificationStatus};

use super::SMCQueryVerification;

use VerificationStatus::*;

use crate::log;

#[derive(Debug, Clone)]
pub struct ProbabilityFloatComparison {
    pub target_probability : f64,
    pub false_positives : f64,
    pub false_negatives : f64,
    pub indifference_up : f64,
    pub indifference_down : f64,
    pub p0 : f64,
    pub p1 : f64,
    pub bound_h0 : f64,
    pub bound_h1 : f64,
    pub current_ratio : f64,
    pub status : VerificationStatus,
    pub runs_executed : usize
}

// Tests if P(Phi) >= p
impl ProbabilityFloatComparison {

    pub fn new(
        target_probability : f64, 
        false_positives : f64, false_negatives : f64, 
        indifference_up : f64, indifference_down : f64
    ) -> Self {
        ProbabilityFloatComparison {
            target_probability,
            false_positives,
            false_negatives,
            indifference_up ,
            indifference_down,
            p0 : target_probability + indifference_up,
            p1 : target_probability - indifference_down,
            bound_h0 : (false_negatives / (1.0 - false_positives)).ln(),
            bound_h1 : ((1.0 - false_negatives) / false_positives).ln(),
            current_ratio : 0.0,
            status : VerificationStatus::Maybe,
            runs_executed : 0
        }
    }

}

impl SMCQueryVerification for ProbabilityFloatComparison {

    fn prepare(&self) {
        log::continue_info("Type : Probability comparison");
        log::continue_info(format!("Comparing : P >= {}", self.target_probability));
        log::continue_info(format!("Allowed false positives : {}%", self.false_positives * 100.0));
        log::continue_info(format!("Allowed false negatives : {}%", self.false_negatives * 100.0));
        log::continue_info(format!("Indifference region : [{},{}]", self.p1, self.p0));
    }

    fn finish(&self) {
        log::continue_info(format!("Runs executed : [{}]", self.runs_executed));
    }

    fn handle_run_result(&mut self, result : VerificationStatus) {
        self.current_ratio += match result {
            Verified => (self.p1 / self.p0).ln(),
            Unverified => ((1.0 - self.p1) / (1.0 - self.p0)).ln(),
            _ => 0.0
        };
        if self.current_ratio <= self.bound_h0 {
            self.status = Verified;
        } else if self.current_ratio >= self.bound_h1 {
            self.status = Unverified;
        }
        self.runs_executed += 1;
    }

    fn get_result(&self) -> SolverResult {
        SolverResult::BoolResult(self.status.good())
    }

    fn must_do_another_run(&self) -> bool {
        self.runs_executed == 0 || self.status.unsure()
    }

}

