use std::{collections::{hash_map::DefaultHasher, HashSet}, hash::{Hash, Hasher}, ops::Not};

use crate::{models::{Label, Model}, solution::{get_problem_type, ProblemType}};

use super::{verifier::Verifiable, EvaluationState, VerificationBound, VerificationStatus};
use serde::{Deserialize, Serialize};
use VerificationStatus::*;

// TODO: Might be useless to include both L and G
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropositionType {
    EQ, NE, LE, GE, LS, GS
}

use PropositionType::*;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expr {
    Name(Label),
    Object(usize),
    Constant(i32),
    ClockComparison(PropositionType, usize, i32),
    Plus(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Negative(Box<Expr>)
}

use Expr::*;

impl Expr {

    pub fn evaluate(&self, state : &impl Verifiable) -> i32 {
        match self {
            Name(_) => panic!("This query contains non-translated names, unable to evaluate !"),
            Constant(i) => *i,
            Object(id) => state.evaluate_object(*id),
            ClockComparison(prop_type, clock, value) => match prop_type {
                EQ => (state.evaluate_clock(*clock) == (*value as f64)) as i32,
                NE => (state.evaluate_clock(*clock) != (*value as f64)) as i32,
                LE => (state.evaluate_clock(*clock) <= (*value as f64)) as i32,
                GE => (state.evaluate_clock(*clock) >= (*value as f64)) as i32,
                LS => (state.evaluate_clock(*clock) < (*value as f64)) as i32,
                GS => (state.evaluate_clock(*clock) > (*value as f64)) as i32,
            }
            Plus(e1, e2) => e1.evaluate(state) + e2.evaluate(state),
            Minus(e1, e2) => e1.evaluate(state) - e2.evaluate(state),
            Multiply(e1, e2) => e1.evaluate(state) * e2.evaluate(state),
            Negative(e) => -e.evaluate(state),
        }
    }

    pub fn contains_clock_proposition(&self) -> bool {
        match self {
            Plus(e1,e2) | 
            Minus(e1, e2) | 
            Multiply(e1,e2)
                => e1.contains_clock_proposition() || e2.contains_clock_proposition(),
            Negative(e) => e.contains_clock_proposition(),
            ClockComparison(_,_,_) => true,
            _ => false,
        }
    }

    // Translate Name(x) to Object(m[x])
    pub fn apply_to_model(&self, model : &impl Model) -> Expr {
        match self {
            Name(x) => Object(model.map_label_to_var(x).unwrap()),
            Plus(e1, e2) => Plus(
                Box::new(e1.apply_to_model(model)), Box::new(e2.apply_to_model(model))
            ),
            Minus(e1, e2) => Minus(
                Box::new(e1.apply_to_model(model)), Box::new(e2.apply_to_model(model))
            ),
            Multiply(e1, e2) => Multiply(
                Box::new(e1.apply_to_model(model)), Box::new(e2.apply_to_model(model))
            ),
            Negative(e) => Negative(Box::new(e.apply_to_model(model))),
            _ => self.clone()
        }
    }

    pub fn accept(&self, visitor : &impl QueryVisitor) {
        match self {
            Plus(e1, e2) |
            Minus(e1, e2) |
            Multiply(e1, e2) => {
                visitor.visit_expression(self);
                e1.accept(visitor);
                e2.accept(visitor);
            },
            _ => visitor.visit_expression(self)
        }
    }

}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Condition {
    True,
    False,
    Deadlock,
    Evaluation(Expr),
    Proposition(PropositionType, Expr, Expr),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
    Implies(Box<Condition>, Box<Condition>),
    Next(Box<Condition>),
    Until(Box<Condition>, Box<Condition>),
}

use Condition::*;

impl Condition {

    pub fn contains_until(&self) -> bool {
        match self {
            Until(_, _) => true,
            Not(c) | Next(c) => c.contains_until(),
            And(c1,c2) | 
            Or(c1, c2) | 
            Implies(c1, c2)
                => c1.contains_until() || c2.contains_until(),
            _ => false
        }
    }

    pub fn is_pure(&self) -> bool {
        match self {
            Until(_, _) => false,
            Next(_) => false,
            Not(c) => c.is_pure(),
            And(c1,c2) | 
            Or(c1, c2) | 
            Implies(c1, c2)
                => c1.is_pure() && c2.is_pure(),
            _ => true
        }
    }

    pub fn contains_clock_proposition(&self) -> bool {
        match self {
            Next(c) | Not(c) => c.contains_clock_proposition(),
            And(c1,c2) | 
            Or(c1, c2) | 
            Until(c1, c2) |
            Implies(c1, c2)
                => c1.contains_clock_proposition() || c2.contains_clock_proposition(),
            Evaluation(e) => e.contains_clock_proposition(),
            Proposition(_, e1, e2) => e1.contains_clock_proposition() || e2.contains_clock_proposition(),
            _ => false
        }
    }

    pub fn apply_to_model(&self, model : &impl Model) -> Condition {
        match self {
            Evaluation(e) => Evaluation(e.apply_to_model(model)),
            Proposition(p_type, e1, e2) => Proposition(
                *p_type, e1.apply_to_model(model), e2.apply_to_model(model)
            ),
            And(c1, c2) => And(
                Box::new(c1.apply_to_model(model)), Box::new(c2.apply_to_model(model))
            ),
            Or(c1, c2) => Or(
                Box::new(c1.apply_to_model(model)), Box::new(c2.apply_to_model(model))
            ),
            Not(c) => Not(Box::new(c.apply_to_model(model))),
            Implies(c1, c2) => Implies(
                Box::new(c1.apply_to_model(model)), Box::new(c2.apply_to_model(model))
            ),
            Next(c) => Next(Box::new(c.apply_to_model(model))),
            Until(c1, c2) => Until(
                Box::new(c1.apply_to_model(model)), Box::new(c2.apply_to_model(model))
            ),
            _ => self.clone()
        }
    }

    pub fn evaluate(&self, state : &impl Verifiable) -> (VerificationStatus, Option<Condition>) {
        match self {
            True => (Verified, None),
            False => (Unverified, None),
            Deadlock => {
                if state.is_deadlocked() {
                    (Verified, None)
                } else {
                    (Unverified, None)
                }
            },
            Evaluation(e) => {
                if e.evaluate(state) > 0 {
                    (Verified, None)
                } else {
                    (Unverified, None)
                }
            },
            Proposition(t, e1, e2) => {
                let res1 = e1.evaluate(state);
                let res2 = e2.evaluate(state);
                let prop_res = match t {
                    EQ => res1 == res2,
                    NE => res1 != res2,
                    LE => res1 <= res2,
                    GE => res1 >= res2,
                    LS => res1 < res2,
                    GS => res1 > res2,
                };
                if prop_res {
                    (Verified, None)
                } else {
                    (Unverified, None)
                }
            },
            And(c1, c2) => { 
                let res1 = c1.evaluate(state);
                let res2 = c2.evaluate(state);
                let status = res1.0 & res2.0;
                match status {
                    Maybe => (Maybe, match (res1.1, res2.1) {
                        (None, None) => None,
                        (Some(c), None) => Some(c),
                        (None, Some(c)) => Some(c),
                        (Some(sub_c1), Some(sub_c2)) => Some(And(Box::new(sub_c1), Box::new(sub_c2))),
                    }),
                    _ => (status, None),
                }
                
            },
            Or(c1, c2) => {
                let res1 = c1.evaluate(state);
                let res2 = c2.evaluate(state);
                let status = res1.0 | res2.0;
                match status {
                    Maybe => (Maybe, match (res1.1, res2.1) {
                        (None, None) => None,
                        (Some(c), None) => Some(c),
                        (None, Some(c)) => Some(c),
                        (Some(sub_c1), Some(sub_c2)) => Some(Or(Box::new(sub_c1), Box::new(sub_c2))),
                    }),
                    _ => (status, None),
                }
            },
            Not(c) => {
                let (status, sub_c) = c.evaluate(state);
                let status = !status;
                match status {
                    Maybe => (Maybe, Some(Not(Box::new(sub_c.unwrap())))),
                    _ => (status, None),
                }
            },
            Implies(c1, c2) => {
                let res1 = c1.evaluate(state);
                let res2 = c2.evaluate(state);
                let status = (!res1.0) | res2.0;
                match status {
                    Maybe => (Maybe, match (res1.1, res2.1) {
                        (None, None) => None,
                        (Some(c), None) => Some(Not(Box::new(c))),
                        (None, Some(c)) => Some(c),
                        (Some(sub_c1), Some(sub_c2)) => Some(
                            Or(
                                Box::new(Not(Box::new(sub_c1))),
                                Box::new(sub_c2)
                            ))
                    }),
                    _ => (status, None)
                }
            },
            Next(c1) => (Maybe, Some(*c1.clone())),
            Until(c1, c2) => {
                let res1 = c1.evaluate(state);
                let res2 = c2.evaluate(state);
                match (res1.0, res2.0) {
                    (_, Verified) => (Verified, None),
                    (Unverified, Unverified) => (Unverified, None),
                    (Verified, Unverified) => (Maybe, Some(self.clone())),
                    (Maybe, Unverified) => (Maybe, Some(
                        And(
                            Box::new(res1.1.unwrap()),
                            Box::new(self.clone())
                        ))),
                    (Maybe, Maybe) => (Maybe, Some(
                        Or(
                            Box::new(res2.1.unwrap()),
                            Box::new(And(
                                Box::new(res1.1.unwrap()),
                                Box::new(self.clone())
                            ))
                        ))),
                    (Unverified, Maybe) => (Maybe, Some(res2.1.unwrap())),
                    (Verified, Maybe) => (Maybe, Some(Or(
                            Box::new(res2.1.unwrap()),
                            Box::new(self.clone())
                        )))
                }
            }
        }
    }

    pub fn accept(&self, visitor : &impl QueryVisitor) {
        match self {
            Not(c) | Next(c) => {
                visitor.visit_condition(self);
                c.accept(visitor);
            },
            And(c1,c2) | 
            Or(c1, c2) | 
            Until(c1, c2) |
            Implies(c1, c2)
                => {
                    visitor.visit_condition(self);
                    c1.accept(visitor);
                    c2.accept(visitor);
                },
            Evaluation(e) => {
                visitor.visit_condition(self);
                e.accept(visitor);
            },
            Proposition(_, e1, e2) => {
                visitor.visit_condition(self);
                e1.accept(visitor);
                e2.accept(visitor);
            },
            _ => visitor.visit_condition(self)
        }
    }

}

impl Not for Condition {
    type Output = Self;
    fn not(self) -> Self::Output {
        Not(Box::new(self))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quantifier {
    #[serde(rename="E")]
    Exists,
    #[serde(rename="A")]
    ForAll,
    #[serde(rename="P")]
    Probability
}

use Quantifier::*;

impl Not for Quantifier {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Exists => Self::ForAll,
            Self::ForAll => Self::Exists,
            Self::Probability => Self::Probability,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn apply_to_model(&mut self, model : &impl Model) {
        self.condition = self.condition.apply_to_model(model);
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