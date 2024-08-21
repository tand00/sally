
use crate::{models::{class_graph::ClassGraph, lbl, model_context::ModelContext, ModelObject}, verification::{Verifiable, VerificationStatus}};

use super::{Solution, SolutionMeta, SolverResult, REACHABILITY};

use crate::log::*;

pub struct ClassGraphReachability;

impl ClassGraphReachability {

    pub fn new() -> Self {
        ClassGraphReachability {}
    }

}

impl Solution for ClassGraphReachability {

    fn get_meta(&self) -> SolutionMeta {
        SolutionMeta {
            name : lbl("ClassGraphReachability"),
            description : String::from("Test a state reachability query against a class graph"),
            problem_type : REACHABILITY,
            model_name : lbl("ClassGraph"),
            result_type : lbl("bool"),
        }
    }

    fn is_compatible(&self, _model : &dyn ModelObject, _ : &ModelContext, query : &crate::verification::query::Query) -> bool {
        (!query.condition.contains_clock_proposition()) && (query.condition.is_state_condition())
    }

    fn solve(&self, model : &dyn ModelObject, _ : &ModelContext, query : &crate::verification::query::Query) -> SolverResult {
        pending("Solving reachability problem on Class graph...");
        let cg : Option<&ClassGraph> = model.as_any().downcast_ref();
        if cg.is_none() {
            return SolverResult::SolverError;
        }
        let cg = cg.unwrap();
        for class in cg.classes.iter() {
            let (status, _) = query.condition.evaluate(class.as_verifiable());
            if status == VerificationStatus::Verified {
                positive("Valid class found !");
                return SolverResult::BoolResult(true);
            }
        }
        negative("No valid class found in the graph");
        SolverResult::BoolResult(false)
    }

}
