use std::{fmt::Display, hash::Hash, ops::Not};

use crate::{models::{Label, Model}, QueryVisitor};

use crate::verification::{Verifiable, VerificationStatus};
use serde::{Deserialize, Serialize};
use VerificationStatus::*;

#[derive(Debug, Clone)]
pub struct MappingError(pub Label);
impl Display for MappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mapping error : label {} not found in context", self.0)
    }
}

pub type MappingResult<T> = Result<T, MappingError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ModelVar {
    pub name : Label,
    #[serde(skip)]
    pub index : Option<usize>,
}

impl ModelVar {
    pub fn name(name : Label) -> ModelVar {
        ModelVar { name, index : None }
    }
    pub fn index(index : usize) -> ModelVar {
        ModelVar { name : Label::new(), index : Some(index) }
    }
    pub fn is_mapped(&self) -> bool {
        self.index.is_some()
    }
    pub fn get_index(&self) -> usize {
        self.index.unwrap()
    }
    pub fn apply_to_model(self, model : &impl Model) -> MappingResult<ModelVar> {
        let res = model.map_label_to_var(&self.name);
        match res {
            None => Err(MappingError(Label::from("Unable to map var to index !"))),
            Some(i) => Ok(ModelVar { name : self.name, index : Some(i)})
        }
    }
    pub fn evaluate(&self, state : &impl Verifiable) -> i32 {
        if self.index.is_none() {
            panic!("Can't evaluate unmapped var !");
        }
        state.evaluate_object(self.get_index())
    }
    pub fn set(&self, state : &mut ModelState, value : i32) {
        if self.index.is_none() {
            panic!("Can't set unmapped var !");
        }
        state.discrete[self.index.unwrap()] = value;
    }
}

impl<T : Into<String>> From<T> for ModelVar {
    fn from(value: T) -> Self {
        ModelVar::name(Label::from(value))
    }
}

pub fn var(name : &str) -> ModelVar {
    ModelVar::name(Label::from(name))
}

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
    ClockComparison(PropositionType, usize, i32),
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
    pub fn apply_to_model(&self, model : &impl Model) -> MappingResult<Expr> {
        match self {
            Var(x) => Ok(Var(x.clone().apply_to_model(model)?)),
            Plus(e1, e2) => Ok(Plus(
                Box::new(e1.apply_to_model(model)?), Box::new(e2.apply_to_model(model)?)
            )),
            Minus(e1, e2) => Ok(Minus(
                Box::new(e1.apply_to_model(model)?), Box::new(e2.apply_to_model(model)?)
            )),
            Multiply(e1, e2) => Ok(Multiply(
                Box::new(e1.apply_to_model(model)?), Box::new(e2.apply_to_model(model)?)
            )),
            Modulo(e1, e2) => Ok(Modulo(
                Box::new(e1.apply_to_model(model)?), Box::new(e2.apply_to_model(model)?)
            )),
            Pow(e1, e2) => Ok(Pow(
                Box::new(e1.apply_to_model(model)?), Box::new(e2.apply_to_model(model)?)
            )),
            Negative(e) => Ok(Negative(Box::new(e.apply_to_model(model)?))),
            _ => Ok(self.clone())
        }
    }

    pub fn accept(&self, visitor : &impl QueryVisitor) {
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

use super::ModelState;

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

    pub fn apply_to_model(&self, model : &impl Model) -> MappingResult<Condition> {
        match self {
            Evaluation(e) => Ok(Evaluation(e.apply_to_model(model)?)),
            Proposition(p_type, e1, e2) => Ok(Proposition(
                *p_type, e1.apply_to_model(model)?, e2.apply_to_model(model)?
            )),
            And(c1, c2) => Ok(And(
                Box::new(c1.apply_to_model(model)?), Box::new(c2.apply_to_model(model)?)
            )),
            Or(c1, c2) => Ok(Or(
                Box::new(c1.apply_to_model(model)?), Box::new(c2.apply_to_model(model)?)
            )),
            Not(c) => Ok(Not(Box::new(c.apply_to_model(model)?))),
            Implies(c1, c2) => Ok(Implies(
                Box::new(c1.apply_to_model(model)?), Box::new(c2.apply_to_model(model)?)
            )),
            Next(c) => Ok(Next(Box::new(c.apply_to_model(model)?))),
            Until(c1, c2) => Ok(Until(
                Box::new(c1.apply_to_model(model)?), Box::new(c2.apply_to_model(model)?)
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

    pub fn is_true(&self, state : &impl Verifiable) -> bool {
        self.evaluate(state).0.good()
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