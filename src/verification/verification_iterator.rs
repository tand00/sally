use std::collections::{HashMap, HashSet};

use crate::models::{Model, ModelState};

use super::{query::Query, EvaluationState};

pub trait VerificationIterator : Iterator {

    fn prepare<T : Model>(&mut self, query : Query, model : &T, initial : ModelState);

}

pub trait SearchStrategy {


    
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepthFirst;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreadthFirst;

impl SearchStrategy for DepthFirst {}
impl SearchStrategy for BreadthFirst {}

pub struct GraphTraversal {
    pub search_strategy : Box<dyn SearchStrategy>,
    evaluation_store : HashSet<EvaluationState>,
    query_store : HashMap<u64, Query>,
}

impl GraphTraversal {



}