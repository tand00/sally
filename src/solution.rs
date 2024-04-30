pub mod class_graph_reachability_synthesis;
pub use class_graph_reachability_synthesis::ClassGraphReachabilitySynthesis;
pub mod class_graph_reachability;
pub use class_graph_reachability::ClassGraphReachability;

use std::any::Any;

use crate::flag;
use crate::models::{lbl, Label, ModelState};
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

pub fn has_problem_type(problem : ProblemType, p_type : ProblemType) -> bool {
    (problem & p_type) > 0
}

pub fn get_problem_type(quantifier : Quantifier, logic : StateLogic) -> ProblemType {
    match (quantifier, logic) {
        (ForAll, Finally) => LIVENESS,
        (ForAll, Globally) => SAFETY,
        (Exists, Finally) => REACHABILITY,
        (Exists, Globally) => PRESERVABILITY,
        _ => UNCLASSIFIED_PROBLEM
    }
}

pub fn problem_label(problem : ProblemType) -> Label {
    let mut characteritics : Vec<&str> = Vec::new(); 
    if problem == 0 {
        return lbl("()");
    }
    if has_problem_type(problem, LIVENESS) {
        characteritics.push("Liveness(AF)");
    }
    if has_problem_type(problem, REACHABILITY) {
        characteritics.push("Reachability(EF)");
    }
    if has_problem_type(problem, PRESERVABILITY) {
        characteritics.push("Preservability(EG)");
    }
    if has_problem_type(problem, SAFETY) {
        characteritics.push("Safety(AG)");
    }
    if has_problem_type(problem, BOUNDEDNESS) {
        characteritics.push("Boundedness");
    }
    if has_problem_type(problem, SYNTHESIS) {
        characteritics.push("Synthesis");
    }
    if has_problem_type(problem, TWO_PLAYERS) {
        characteritics.push("TwoPlayers");
    }
    Label::from(characteritics.join("|"))
}

#[derive(Debug, Clone, PartialEq)]
pub enum SolverResult {
    SolverError,
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