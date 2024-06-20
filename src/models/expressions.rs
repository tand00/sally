use std::{collections::HashSet, hash::Hash, ops::Not};

use crate::QueryVisitor;

use crate::verification::{Verifiable, VerificationStatus};
use serde::{Deserialize, Serialize};
use VerificationStatus::*;

// TODO: Might be useless to include both L and G
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropositionType {
    EQ, NE, LE, GE, LS, GS
}

use PropositionType::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expr {
    Var(ModelVar),
    Constant(i32),
    ClockComparison(PropositionType, ModelClock, i32),
    Plus(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Negative(Box<Expr>),
    Modulo(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>)
}

use Expr::*;

impl Expr {

    pub fn evaluate(&self, state : &impl Verifiable) -> i32 {
        match self {
            Constant(i) => *i,
            Var(x) => x.evaluate(state),
            ClockComparison(prop_type, clock, value) => match prop_type {
                EQ => (state.evaluate_clock(clock) == (*value as f64)) as i32,
                NE => (state.evaluate_clock(clock) != (*value as f64)) as i32,
                LE => (state.evaluate_clock(clock) <= (*value as f64)) as i32,
                GE => (state.evaluate_clock(clock) >= (*value as f64)) as i32,
                LS => (state.evaluate_clock(clock) < (*value as f64)) as i32,
                GS => (state.evaluate_clock(clock) > (*value as f64)) as i32,
            }
            Plus(e1, e2) => e1.evaluate(state) + e2.evaluate(state),
            Minus(e1, e2) => e1.evaluate(state) - e2.evaluate(state),
            Multiply(e1, e2) => e1.evaluate(state) * e2.evaluate(state),
            Negative(e) => -e.evaluate(state),
            Modulo(e1, e2) => e1.evaluate(state) % e2.evaluate(state),
            Pow(e1, e2) => e1.evaluate(state).pow(e2.evaluate(state) as u32)
        }
    }

    pub fn contains_clock_proposition(&self) -> bool {
        match self {
            Plus(e1,e2) | 
            Minus(e1, e2) | 
            Multiply(e1,e2) |
            Modulo(e1,e2) |
            Pow(e1, e2)
                => e1.contains_clock_proposition() || e2.contains_clock_proposition(),
            Negative(e) => e.contains_clock_proposition(),
            ClockComparison(_,_,_) => true,
            _ => false,
        }
    }

    // Translate Name(x) to Object(m[x])
    pub fn apply_to(&self, ctx : &ModelContext) -> MappingResult<Expr> {
        match self {
            Var(x) => Ok(Var(x.apply_to(ctx)?)),
            Plus(e1, e2) => Ok(Plus(
                Box::new(e1.apply_to(ctx)?), Box::new(e2.apply_to(ctx)?)
            )),
            Minus(e1, e2) => Ok(Minus(
                Box::new(e1.apply_to(ctx)?), Box::new(e2.apply_to(ctx)?)
            )),
            Multiply(e1, e2) => Ok(Multiply(
                Box::new(e1.apply_to(ctx)?), Box::new(e2.apply_to(ctx)?)
            )),
            Modulo(e1, e2) => Ok(Modulo(
                Box::new(e1.apply_to(ctx)?), Box::new(e2.apply_to(ctx)?)
            )),
            Pow(e1, e2) => Ok(Pow(
                Box::new(e1.apply_to(ctx)?), Box::new(e2.apply_to(ctx)?)
            )),
            Negative(e) => Ok(Negative(Box::new(e.apply_to(ctx)?))),
            _ => Ok(self.clone())
        }
    }

    pub fn accept(&self, visitor : &mut impl QueryVisitor) {
        match self {
            Plus(e1, e2) |
            Minus(e1, e2) |
            Multiply(e1, e2) |
            Modulo(e1, e2) |
            Pow(e1, e2)
                => {
                visitor.visit_expression(self);
                e1.accept(visitor);
                e2.accept(visitor);
            },
            _ => visitor.visit_expression(self)
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

use super::{model_clock::ModelClock, model_context::ModelContext, model_var::{MappingResult, ModelVar}};

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

    pub fn is_state_condition(&self) -> bool {
        match self {
            Until(_, _) => false,
            Next(_) => false,
            Not(c) => c.is_state_condition(),
            And(c1,c2) | 
            Or(c1, c2) | 
            Implies(c1, c2)
                => c1.is_state_condition() && c2.is_state_condition(),
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

    pub fn apply_to(&self, ctx : &ModelContext) -> MappingResult<Condition> {
        match self {
            Evaluation(e) => Ok(Evaluation(e.apply_to(ctx)?)),
            Proposition(p_type, e1, e2) => Ok(Proposition(
                *p_type, e1.apply_to(ctx)?, e2.apply_to(ctx)?
            )),
            And(c1, c2) => Ok(And(
                Box::new(c1.apply_to(ctx)?), Box::new(c2.apply_to(ctx)?)
            )),
            Or(c1, c2) => Ok(Or(
                Box::new(c1.apply_to(ctx)?), Box::new(c2.apply_to(ctx)?)
            )),
            Not(c) => Ok(Not(Box::new(c.apply_to(ctx)?))),
            Implies(c1, c2) => Ok(Implies(
                Box::new(c1.apply_to(ctx)?), Box::new(c2.apply_to(ctx)?)
            )),
            Next(c) => Ok(Next(Box::new(c.apply_to(ctx)?))),
            Until(c1, c2) => Ok(Until(
                Box::new(c1.apply_to(ctx)?), Box::new(c2.apply_to(ctx)?)
            )),
            _ =>Ok(self.clone())
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

    pub fn accept(&self, visitor : &mut impl QueryVisitor) {
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

    pub fn is_true(&self, state : &impl Verifiable) -> bool {
        self.evaluate(state).0.good()
    }

    pub fn get_objects(&self) -> ObjectsScannerVisitor {
        let mut visitor = ObjectsScannerVisitor::new();
        self.accept(&mut visitor);
        visitor
    }

}

impl Default for Condition {
    fn default() -> Self {
        True
    }
}

impl Not for Condition {
    type Output = Self;
    fn not(self) -> Self::Output {
        Not(Box::new(self))
    }
}

pub struct ObjectsScannerVisitor {
    pub vars : HashSet<ModelVar>,
    pub clocks : HashSet<ModelClock>
}
impl ObjectsScannerVisitor {
    pub fn new() -> Self {
        ObjectsScannerVisitor { 
            vars : HashSet::new(),
            clocks : HashSet::new()
        }
    }
}
impl QueryVisitor for ObjectsScannerVisitor {
    fn visit_query(&mut self, _query : &crate::Query) { }
    fn visit_condition(&mut self, _condition : &Condition) { }
    fn visit_expression(&mut self, expr : &Expr) {
        if let Var(x) = expr {
            self.vars.insert(x.clone());
        } else if let ClockComparison(_, c, _) = expr {
            self.clocks.insert(c.clone());
        }
    }
}