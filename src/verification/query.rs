use std::{collections::{hash_map::DefaultHasher, HashSet}, hash::{Hash, Hasher}, ops::Not};

use crate::models::time::ClockValue;

use super::{verifier::Verifiable, EvaluationState, VerificationBound, VerificationStatus};
use VerificationStatus::*;

// TODO: Might be useless to include both L and G
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropositionType {
    EQ, NE, LE, GE, LS, GS
}

use PropositionType::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Expr {
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

}

#[derive(Clone, PartialEq, Eq, Hash)]
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

}

impl Not for Condition {
    type Output = Self;
    fn not(self) -> Self::Output {
        Not(Box::new(self))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Quantifier {
    Exists, ForAll
}

use Quantifier::*;

impl Not for Quantifier {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Exists => Self::ForAll,
            Self::ForAll => Self::Exists,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StateLogic {
    Finally, Globally, RawCondition
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

#[derive(Clone, PartialEq, Eq)]
pub struct Query {
    pub quantifier : Quantifier,
    pub logic : StateLogic,
    pub condition : Condition,

    pub total_status : VerificationStatus,
    pub run_status : VerificationStatus,

    pending_conditions : Vec<Condition>,

    pub collapse_subconditions : bool,
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
        }
    }

    pub fn end_run(&mut self) {
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
        };
        match self.logic {
            Finally => Maybe,
            Globally => Verified,
            RawCondition => Maybe,
        };
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

    pub fn get_as_logic(&self, logic : StateLogic) -> Query {
        if self.logic == RawCondition {
            panic!("Can't convert RawCondition to F or G !");
        }
        if self.logic == logic {
            return Query::new(self.quantifier, logic, self.condition.clone());
        }
        return Query::new(!self.quantifier, logic, !self.condition.clone());
    }

    pub fn get_as_quantifier(&self, quantifier : Quantifier) -> Query {
        if self.quantifier == quantifier {
            return Query::new(quantifier, self.logic, self.condition.clone());
        }
        return Query::new(quantifier, !self.logic, !self.condition.clone());
    }

}