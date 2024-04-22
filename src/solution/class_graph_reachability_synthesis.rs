use crate::models::{lbl, Label};

use super::{Solution, SolutionMeta, SolverResult, REACHABILITY, SYNTHESIS, TWO_PLAYERS};

pub struct ClassGraphReachabilitySynthesis {

}

impl ClassGraphReachabilitySynthesis {

    pub fn new() -> Self {
        ClassGraphReachabilitySynthesis {}
    }

}

impl Solution for ClassGraphReachabilitySynthesis {

    fn get_meta(&self) -> SolutionMeta {
        SolutionMeta {
            name : lbl("ClassGraphReachabilitySynthesis"),
            description : String::from("Compute the reachability game strategy for a two players class graph"),
            problem_type : REACHABILITY | SYNTHESIS | TWO_PLAYERS,
            model_name : lbl("ClassGraph"),
            result_type : lbl("Strategy"),
        }
    }

    fn is_compatible(&self, model : &dyn std::any::Any, query : &crate::verification::query::Query) -> bool {
        false
    }

    fn solve(&mut self, model : &dyn std::any::Any, query : &crate::verification::query::Query) -> SolverResult {
        SolverResult::Unsatisfied
    }

}