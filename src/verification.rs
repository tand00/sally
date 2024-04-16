mod verifier;
mod verification_iterator;

pub mod query;

pub use verifier::*;

pub mod decidable_solutions {
    use crate::flag;

    use super::query::{Quantifier, StateLogic};
    pub type DecidableSolution = u16;
    pub const UNCLASSIFIED_PROBLEM : DecidableSolution = 0;
    pub const LIVENESS : DecidableSolution = flag!(0);
    pub const SAFETY : DecidableSolution = flag!(1);
    pub const REACHABILITY : DecidableSolution = flag!(2);
    pub const PRESERVABILITY : DecidableSolution = flag!(3);
    pub const BOUNDEDNESS : DecidableSolution = flag!(4);

    pub fn has_solution(model_characteristics : DecidableSolution, solution : DecidableSolution) -> bool {
        (model_characteristics & solution) != 0
    }

    use super::query::Quantifier::*;
    use super::query::StateLogic::*;

    pub fn get_problem_type(quantifier : Quantifier, logic : StateLogic) -> DecidableSolution {
        match (quantifier, logic) {
            (ForAll, Finally) => LIVENESS,
            (ForAll, Globally) => SAFETY,
            (Exists, Finally) => REACHABILITY,
            (Exists, Globally) => PRESERVABILITY,
            _ => UNCLASSIFIED_PROBLEM
        }
    }
}