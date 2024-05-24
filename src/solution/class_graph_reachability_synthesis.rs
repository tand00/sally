use crate::models::{lbl, model_context::ModelContext, Label};

use super::{Solution, SolutionMeta, SolverResult, REACHABILITY, SYNTHESIS, TWO_PLAYERS};

pub struct ClassGraphReachabilitySynthesis;

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

    fn is_compatible(&self, model : &dyn std::any::Any, ctx : &ModelContext, query : &crate::verification::query::Query) -> bool {
        (!query.condition.contains_clock_proposition()) && (query.condition.is_pure())
    }

    fn solve(&mut self, model : &dyn std::any::Any, ctx : &ModelContext, query : &crate::verification::query::Query) -> SolverResult {
        SolverResult::SolverError
    }

}