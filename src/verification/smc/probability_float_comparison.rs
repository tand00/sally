use crate::{solution::SolverResult, verification::VerificationStatus};

use super::SMCQueryVerification;

use VerificationStatus::*;

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
    pub status : VerificationStatus
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
            bound_h0 : (false_positives / (1.0 - false_positives)).ln(),
            bound_h1 : ((1.0 - false_positives) / false_positives).ln(),
            current_ratio : 0.0,
            status : VerificationStatus::Maybe
        }
    }

}

impl SMCQueryVerification for ProbabilityFloatComparison {

    fn handle_run_result(&mut self, result : VerificationStatus) {
        self.current_ratio += match result {
            Verified => (self.p1 / self.p0).ln(),
            Unverified => ((1.0 - self.p1) / (1.0 - self.p0)).ln(),
            _ => 0.0
        };
        if self.current_ratio <= self.bound_h0 {
            self.status = Verified
        } else if self.current_ratio >= self.bound_h1 {
            self.status = Unverified
        }
    }

    fn get_result(&self) -> SolverResult {
        SolverResult::BoolResult(self.status.good())
    }

    fn must_do_another_run(&self) -> bool {
        self.status.unsure()
    }

}

