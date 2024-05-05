use std::{collections::{hash_map::DefaultHasher, HashSet}, hash::{Hash, Hasher}, ops::Not};

use crate::{models::{expressions::{Condition, Expr, MappingResult}, Model}, solution::{get_problem_type, ProblemType}};

use super::{verifier::Verifiable, EvaluationState, VerificationBound, VerificationStatus};
use serde::{Deserialize, Serialize};
use VerificationStatus::*;

use Condition::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quantifier {
    #[serde(rename="E")]
    Exists,
    #[serde(rename="A")]
    ForAll,
    #[serde(rename="P")]
    Probability,
    LTL
}

use Quantifier::*;

impl Not for Quantifier {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Exists => Self::ForAll,
            Self::ForAll => Self::Exists,
            _ => self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateLogic {
    #[serde(rename="F")]
    Finally, 
    #[serde(rename="G")]
    Globally, 
    #[serde(rename="raw")]
    RawCondition
}

use StateLogic::*;

impl Not for StateLogic {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Finally => Self::Globally,
            Self::Globally => Self::Finally,
            Self::RawCondition => Self::RawCondition
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Query {
    pub quantifier : Quantifier,
    pub logic : StateLogic,
    pub condition : Condition,

    #[serde(skip)]
    pub total_status : VerificationStatus,
    #[serde(skip)]
    pub run_status : VerificationStatus,

    #[serde(skip)]
    pending_conditions : Vec<Condition>,

    #[serde(skip)]
    pub collapse_subconditions : bool,

    pub run_bound : VerificationBound
}

impl Query {

    pub fn new(quantifier : Quantifier, logic : StateLogic, condition : Condition) -> Self {
        Query {
            quantifier,
            logic,
            condition,
            total_status : Maybe,
            run_status : Maybe,
            pending_conditions : Vec::new(),
            collapse_subconditions : false,
            run_bound : VerificationBound::NoRunBound
        }
    }

    pub fn end_run(&mut self) {
        self.pending_conditions.clear();
        if self.run_status == Maybe {
            self.run_status = match self.logic {
                Finally => Unverified,
                Globally => Verified,
                RawCondition => Unverified
            }
        }
        match self.quantifier {
            Exists => self.total_status |= self.run_status,
            ForAll => self.total_status &= self.run_status,
            _ => ()
        };
        if self.is_decided() {
            self.end_verification();
        }
    }

    pub fn reset_run(&mut self) {
        self.run_status = Maybe;
    }

    pub fn end_verification(&mut self) {
        if self.total_status == Maybe {
            self.total_status = match self.quantifier {
                Exists => Unverified,
                ForAll => Verified,
                _ => Maybe
            }
        }
    }

    pub fn verify_state(&mut self, state : &impl Verifiable) {
        let mut finished = false;
        let mut new_pendings : HashSet<Condition> = HashSet::new(); // Hashset to prevent propagation of Until
        let mut pending = Some(self.condition.clone());
        while pending.is_some() && !finished {
            let (res, follow) = pending.unwrap().evaluate(state);
            match res {
                Maybe => { new_pendings.insert(follow.unwrap()); },
                _ => finished = self.process_result(res)
            }
            pending = self.pending_conditions.pop();
        }
        if finished {
            self.end_run();
            return;
        }
        if self.collapse_subconditions && new_pendings.len() > 0 {
            self.pending_conditions = vec![self.collapse_conditions(new_pendings)];
        } else {
            self.pending_conditions = Vec::from_iter(new_pendings);
        }
    }

    fn process_result(&mut self, result : VerificationStatus) -> bool {
        match self.logic {
            Finally => self.run_status |= result,
            Globally => self.run_status &= result,
            RawCondition => self.run_status = result,
        };
        match self.run_status {
            Maybe => false,
            _ => true,
        }
    }

    fn collapse_conditions(&self, set : HashSet<Condition>) -> Condition {
        let mut new_conditions = set.iter();
        let mut collapsed = new_conditions.next().cloned().unwrap();
        for c in new_conditions.cloned() {
            match self.logic {
                Finally => collapsed = Or(Box::new(collapsed), Box::new(c)),
                _ => collapsed = And(Box::new(collapsed), Box::new(c)),
            }
        }
        collapsed
    }

    pub fn get_evaluation_state(&self, state : &impl Verifiable) -> EvaluationState {
        let mut s = DefaultHasher::new();
        self.pending_conditions.hash(&mut s);
        state.hash(&mut s);
        s.finish()
    }

    pub fn get_progress_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.pending_conditions.hash(&mut s);
        s.finish()
    }

    pub fn complement(self) -> Query {
        return Query::new(!self.quantifier, !self.logic, !self.condition.clone());
    }

    pub fn is_run_decided(&self) -> bool {
        match self.run_status {
            Maybe => false,
            _ => true
        }
    }

    pub fn is_decided(&self) -> bool {
        match self.total_status {
            Maybe => false,
            _ => true
        }
    }

    pub fn problem_type(&self) -> ProblemType {
        get_problem_type(self.quantifier, self.logic)
    }

    pub fn apply_to_model(&mut self, model : &impl Model) -> MappingResult<()> {
        self.condition = self.condition.apply_to_model(model)?;
        Ok(())
    }

    pub fn accept_visitor(&self, visitor : &impl QueryVisitor) {
        visitor.visit_query(self);
        self.condition.accept(visitor);
    }

}

pub trait QueryVisitor {

    fn visit_query(&self, query : &Query);
    fn visit_condition(&self, condition : &Condition);
    fn visit_expression(&self, expr : &Expr);

}