pub mod class_graph_reachability_synthesis;
pub use class_graph_reachability_synthesis::ClassGraphReachabilitySynthesis;

use std::any::Any;

use crate::flag;
use crate::models::{Label, ModelState};
use crate::verification::query::{Quantifier, Query, StateLogic};
use Quantifier::*;
use StateLogic::*;

pub type ProblemType = u16;
pub const UNCLASSIFIED_PROBLEM : ProblemType = 0;
pub const LIVENESS : ProblemType = flag!(0);
pub const SAFETY : ProblemType = flag!(1);
pub const REACHABILITY : ProblemType = flag!(2);
pub const PRESERVABILITY : ProblemType = flag!(3);
pub const BOUNDEDNESS : ProblemType = flag!(4);
pub const SYNTHESIS : ProblemType = flag!(5);
pub const TWO_PLAYERS : ProblemType = flag!(6);

pub fn get_problem_type(quantifier : Quantifier, logic : StateLogic) -> ProblemType {
    match (quantifier, logic) {
        (ForAll, Finally) => LIVENESS,
        (ForAll, Globally) => SAFETY,
        (Exists, Finally) => REACHABILITY,
        (Exists, Globally) => PRESERVABILITY,
        _ => UNCLASSIFIED_PROBLEM
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SolverResult {
    Unsatisfied,
    BoolResult(bool),
    IntResult(i32),
    FloatResult(f64),
    StateResult(ModelState),
    TraceResult(Vec<Label>),
    StrategyResult,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SolutionMeta {
    pub name : Label,
    pub description : String,
    pub problem_type : ProblemType,
    pub model_name : Label,
    pub result_type : Label,
}

pub trait Solution {

    fn get_meta(&self) -> SolutionMeta;

    fn is_compatible(&self, model : &dyn Any, query : &Query) -> bool;

    fn solve(&mut self, model : &dyn Any, query : &Query) -> SolverResult;

}