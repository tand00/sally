use crate::{models::{class_graph::ClassGraph, lbl, model_context::ModelContext}, verification::{Verifiable, VerificationStatus}};

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
            description : String::from("Test a pure reachability query against a class graph"),
            problem_type : REACHABILITY,
            model_name : lbl("ClassGraph"),
            result_type : lbl("bool"),
        }
    }

    fn is_compatible(&self, _model : &dyn std::any::Any, ctx : &ModelContext, query : &crate::verification::query::Query) -> bool {
        (!query.condition.contains_clock_proposition()) && (query.condition.is_pure())
    }

    fn solve(&mut self, model : &dyn std::any::Any, ctx : &ModelContext, query : &crate::verification::query::Query) -> SolverResult {
        pending("Solving reachability problem on Class graph...");
        let cg : Option<&ClassGraph> = model.downcast_ref();
        if cg.is_none() {
            return SolverResult::SolverError;
        }
        let cg = cg.unwrap();
        for class in cg.classes.iter() {
            let (status, _) = query.condition.evaluate(class.borrow().as_verifiable());
            if status == VerificationStatus::Verified {
                positive("Valid class found !");
                return SolverResult::BoolResult(true);
            }
        }
        negative("No valid class found in the graph");
        SolverResult::BoolResult(false)
    }

}