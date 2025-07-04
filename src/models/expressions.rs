use std::fmt::Display;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use std::{collections::HashSet, hash::Hash, ops::Not};

use crate::verification::query::{Query, QueryVisitor};

use crate::verification::{Verifiable, VerificationStatus};
use serde::{Deserialize, Serialize};
use VerificationStatus::*;

// TODO: Might be useless to include both L and G
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropositionType {
    EQ, NE, LE, GE, LS, GS
}

impl Display for PropositionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EQ => write!(f, "=="),
            NE => write!(f, "!="),
            LE => write!(f, "<="),
            GE => write!(f, ">="),
            LS => write!(f, "<"),
            GS => write!(f, ">"),
        }
    }
}

impl Not for PropositionType {
    type Output = PropositionType;

    fn not(self) -> Self::Output {
        match self {
            EQ => NE,
            NE => EQ,
            LE => GS,
            GE => LS,
            LS => GE,
            GS => LE
        }
    }
}

use PropositionType::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expr {
    Var(ModelVar),
    Constant(i32),
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
            Plus(e1, e2) => e1.evaluate(state) + e2.evaluate(state),
            Minus(e1, e2) => e1.evaluate(state) - e2.evaluate(state),
            Multiply(e1, e2) => e1.evaluate(state) * e2.evaluate(state),
            Negative(e) => -e.evaluate(state),
            Modulo(e1, e2) => e1.evaluate(state) % e2.evaluate(state),
            Pow(e1, e2) => e1.evaluate(state).pow(e2.evaluate(state) as u32)
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

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Var(v) => v.fmt(f),
            Constant(i) => i.fmt(f),
            Plus(a, b) => write!(f, "({a} + {b})"),
            Minus(a, b) => write!(f, "({a} - {b})"),
            Multiply(a, b) => write!(f, "({a} * {b})"),
            Negative(x) => write!(f, "-{x}"),
            Modulo(a, b) => write!(f, "({a} % {b})"),
            Pow(a, b) => write!(f, "({a} ^ {b})"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Condition {
    True,
    False,
    Deadlock,
    Evaluation(Expr),
    ClockComparison(PropositionType, ModelClock, i32),
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
            ClockComparison(_,_,_) => true,
            _ => false
        }
    }

    pub fn is_clock_guard(&self) -> bool {
        match self {
            Next(_) | Until(_,_) => false, // Guards are instantaneous
            Not(c) => c.is_clock_guard(),
            And(c1,c2) |
            Or(c1, c2) |
            Implies(c1, c2)
                => c1.is_clock_guard() && c2.is_clock_guard(),
            ClockComparison(_,_,_) => true,
            True | False => true,
            _ => false
        }
    }

    pub fn remove_clock(&self, clock : &ModelClock) -> Condition {
        match self {
            Next(c) | Not(c) => {
                if let ClockComparison(_, ref cond_clock, _) = **c {
                    if cond_clock.get_index() == clock.get_index() {
                        return True;
                    }
                }
                self.clone()
            }
            And(c1, c2) => c1.remove_clock(clock) & c2.remove_clock(clock),
            Or(c1, c2) => c1.remove_clock(clock) | c2.remove_clock(clock),
            Until(c1, c2) => Until(
                Box::new(c1.remove_clock(clock)),
                Box::new(c2.remove_clock(clock))
            ),
            Implies(c1, c2) => Implies(
                Box::new(c1.remove_clock(clock)),
                Box::new(c2.remove_clock(clock))
            ),
            ClockComparison(_, c, _) => {
                if c.get_index() == clock.get_index() {
                    return True;
                }
                self.clone()
            },
            _ => self.clone()
        }
    }

    pub fn apply_to(&self, ctx : &ModelContext) -> MappingResult<Condition> {
        match self {
            Evaluation(e) => Ok(Evaluation(e.apply_to(ctx)?)),
            Proposition(p_type, e1, e2) => Ok(Proposition(
                *p_type, e1.apply_to(ctx)?, e2.apply_to(ctx)?
            )),
            ClockComparison(op, c, v) => {
                Ok(ClockComparison(*op, c.apply_to(ctx)?, *v))
            }
            And(c1, c2) => Ok((c1.apply_to(ctx)?) & (c2.apply_to(ctx)?)),
            Or(c1, c2) => Ok((c1.apply_to(ctx)?) | (c2.apply_to(ctx)?)),
            Not(c) => Ok(!(c.apply_to(ctx)?)),
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
            ClockComparison(prop_type, clock, value) => {
                let prop_res = match prop_type {
                    EQ => state.evaluate_clock(clock) == (*value as f64),
                    NE => state.evaluate_clock(clock) != (*value as f64),
                    LE => state.evaluate_clock(clock) <= (*value as f64),
                    GE => state.evaluate_clock(clock) >= (*value as f64),
                    LS => state.evaluate_clock(clock) < (*value as f64),
                    GS => state.evaluate_clock(clock) > (*value as f64),
                };
                if prop_res {
                    (Verified, None)
                } else {
                    (Unverified, None)
                }
            }
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

    pub fn distribute_not(&self) -> Condition {
        match self { //prop
            Not(c) => {
                let sub = Condition::clone(c);
                match sub {
                    True => False,
                    False => True,
                    Proposition(op, e1, e2) => Proposition(!op, e1, e2),
                    ClockComparison(op, c, v) => ClockComparison(!op, c.clone(), v),
                    And(c1, c2) => Not(c1).distribute_not() | Not(c2).distribute_not(),
                    Or(c1, c2) => Not(c1).distribute_not() & Not(c2).distribute_not(),
                    Next(sub) => Next(Box::new(Not(sub).distribute_not())),
                    Implies(c1, c2) => c1.distribute_not() & Not(c2).distribute_not(),
                    Not(sub) => sub.distribute_not(),
                    //Until ?
                    _ => Not(Box::new(sub.distribute_not()))
                }
            },
            And(c1,c2) => c1.distribute_not() & c2.distribute_not(),
            Or(c1, c2) => c1.distribute_not() | c2.distribute_not(),
            Until(c1, c2) => Until(
                Box::new(c1.distribute_not()),
                Box::new(c2.distribute_not())
            ),
            Implies(c1, c2) => Implies(
                Box::new(c1.distribute_not()),
                Box::new(c2.distribute_not())
            ),
            Next(c) => Next(Box::new(c.distribute_not())),
            _ => self.clone()
        }
    }

    pub fn disjunctive_normal(&self) -> Condition {
        match self {
            Not(c) => {
                let sub = Condition::clone(c);
                match sub {
                    Not(c1) => c1.disjunctive_normal(),
                    And(c1, c2) => Or(
                        Box::new(Not(c1).disjunctive_normal()),
                        Box::new(Not(c2).disjunctive_normal())
                    ),
                    Or(c1, c2) => And(
                        Box::new(Not(c1)),
                        Box::new(Not(c2))
                    ).disjunctive_normal(),
                    Implies(c1, c2) => And(
                        c1, Box::new(Not(c2))
                    ).disjunctive_normal(),
                    Next(c) => Next(Box::new(Not(c))).disjunctive_normal(),
                    Until(a,b) => Not(Box::new(Until(
                        Box::new(a.disjunctive_normal()),
                        Box::new(b.disjunctive_normal())
                    ))),
                    _ => Not(c.clone()).distribute_not()
                }
            }
            Or(c1, c2) => Or(
                Box::new(c1.disjunctive_normal()),
                Box::new(c2.disjunctive_normal())
            ),
            Implies(c1, c2) => Or(
                Box::new(Not(c1.clone()).disjunctive_normal()),
                Box::new(c2.disjunctive_normal())
            ),
            And(c1, c2) => {
                let c1 = c1.disjunctive_normal();
                let c2 = c2.disjunctive_normal();
                match (c1, c2) {
                    (Or(a,b), Or(c,d)) => Or(
                        Box::new(Or(
                            Box::new(And(a.clone(), c.clone()).disjunctive_normal()),
                            Box::new(And(a, d.clone()).disjunctive_normal())
                        )),
                        Box::new(Or(
                            Box::new(And(b.clone(), c).disjunctive_normal()),
                            Box::new(And(b, d).disjunctive_normal())
                        )),
                    ),
                    (Or(a,b), c2) => Or(
                        Box::new(And(
                            a, Box::new(c2.clone())
                        ).disjunctive_normal()),
                        Box::new(And(
                            b, Box::new(c2)
                        ).disjunctive_normal()),
                    ),
                    (c1, Or(c,d)) => Or(
                        Box::new(And(
                            c, Box::new(c1.clone())
                        ).disjunctive_normal()),
                        Box::new(And(
                            d, Box::new(c1)
                        ).disjunctive_normal()),
                    ),
                    (c1,c2) => c1 & c2,
                }
            },
            Next(sub) => {
                let sub = sub.disjunctive_normal();
                match sub {
                    Or(a,b) => Next(a).disjunctive_normal() | Next(b).disjunctive_normal(),
                    And(a,b) => Next(a).disjunctive_normal() & Next(b).disjunctive_normal(),
                    _ => Next(Box::new(sub)) // sub cannot be implies
                }
            },
            Until(a, b) => Until(
                Box::new(a.disjunctive_normal()),
                Box::new(b.disjunctive_normal())
            ),
            _ => self.clone()
        }
    }

    pub fn conjunctive_normal(&self) -> Condition {
        (!self.disjunctive_normal()).distribute_not()
    }

    pub fn conjunctions(&self) -> Vec<Condition> {
        match self {
            Or(a, b) => {
                let mut conjs = a.conjunctions();
                conjs.append(&mut b.conjunctions());
                conjs
            },
            _ => vec![self.clone()]
        }
    }

    pub fn to_greater_eq(&self) -> Condition {
        match self { //prop
            Not(c) => !c.to_greater_eq(),
            And(c1,c2) => c1.to_greater_eq() & c2.to_greater_eq(),
            Or(c1, c2) => c1.to_greater_eq() | c2.to_greater_eq(),
            Until(c1, c2) => Until(
                Box::new(c1.to_greater_eq()),
                Box::new(c2.to_greater_eq())
            ),
            Implies(c1, c2) => Implies(
                Box::new(c1.to_greater_eq()),
                Box::new(c2.to_greater_eq())
            ),
            Next(c) => Next(Box::new(c.to_greater_eq())),
            Proposition(op, e1, e2) => {
                match op {
                    EQ | NE | GE | GS => Proposition(*op, e1.clone(), e2.clone()),
                    LS => Proposition(GS, e2.clone(), e1.clone()),
                    LE => Proposition(GE, e2.clone(), e1.clone())
                }
            },
            Evaluation(e) => Evaluation(e.clone()),
            _ => self.clone()
        }
    }

    pub fn to_lesser_eq(&self) -> Condition {
        match self { //prop
            Not(c) => !c.to_lesser_eq(),
            And(c1,c2) => c1.to_lesser_eq() & c2.to_lesser_eq(),
            Or(c1, c2) => c1.to_lesser_eq() | c2.to_lesser_eq(),
            Until(c1, c2) => Until(
                Box::new(c1.to_lesser_eq()),
                Box::new(c2.to_lesser_eq())
            ),
            Implies(c1, c2) => Implies(
                Box::new(c1.to_lesser_eq()),
                Box::new(c2.to_lesser_eq())
            ),
            Next(c) => Next(Box::new(c.to_lesser_eq())),
            Proposition(op, e1, e2) => {
                match op {
                    EQ | NE | LE | LS => Proposition(*op, e1.clone(), e2.clone()),
                    GS => Proposition(LS, e2.clone(), e1.clone()),
                    GE => Proposition(LE, e2.clone(), e1.clone())
                }
            },
            Evaluation(e) => Evaluation(e.clone()),
            _ => self.clone()
        }
    }

    pub fn is_true(&self, state : &impl Verifiable) -> bool {
        self.evaluate(state).0.good()
    }

    pub fn is_default(&self) -> bool {
        *self == True
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
        match self {
            True => False,
            False => True,
            c => Not(Box::new(c))
        }
    }
}

impl BitAnd for Condition {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (True, True) => True,
            (False, _) | (_, False) => False,
            (True, c) | (c, True) => c,
            (a,b) => And(Box::new(a), Box::new(b))
        }
    }
}

impl BitOr for Condition {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (False, False) => False,
            (True, _) | (_, True) => True,
            (False, c) | (c, False) => c,
            (a,b) => Or(Box::new(a), Box::new(b))
        }
    }
}

impl BitAndAssign for Condition {
    fn bitand_assign(&mut self, rhs: Self) {
        let cond = std::mem::take(self);
        *self = cond & rhs;
    }
}

impl BitOrAssign for Condition {
    fn bitor_assign(&mut self, rhs: Self) {
        let cond = std::mem::take(self);
        *self = cond | rhs;
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            True => write!(f, "true"),
            False => write!(f, "false"),
            Deadlock => write!(f, "deadlock"),
            Evaluation(e) => e.fmt(f),
            ClockComparison(op, c, i) => write!(f, "({c} {op} {i})"),
            Proposition(op, a, b) => write!(f, "({a} {op} {b})"),
            And(a, b) => write!(f, "({a} && {b})"),
            Or(a, b) => write!(f, "({a} || {b})"),
            Not(x) => write!(f, "!{x}"),
            Implies(a, b) => write!(f, "({a} => {b})"),
            Next(x) => write!(f, "(X{x})"),
            Until(a, b) => write!(f, "({a} U {b})"),
        }
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
    fn visit_query(&mut self, _query : &Query) { }
    fn visit_condition(&mut self, condition : &Condition) {
        if let ClockComparison(_, c, _) = condition {
            self.clocks.insert(c.clone());
        }
    }
    fn visit_expression(&mut self, expr : &Expr) {
        if let Var(x) = expr {
            self.vars.insert(x.clone());
        }
    }
}
