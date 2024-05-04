use pest_derive::Parser;
use pest::{iterators::Pairs, pratt_parser::PrattParser, Parser};
use serde::{Deserialize, Serialize};

use crate::models::Label;

use super::query::*;

// Parser for text queries, using Pest for now... Might be fun to build an automata later :) !

#[derive(Debug, Clone, Serialize, Deserialize)] //TODO! maybe delete unnecessary serialization
pub struct QueryParsingError;
pub type QueryParsingResult<T> = Result<T, QueryParsingError>;

#[derive(Parser)]
#[grammar = "verification/query_grammar.pest"]
struct TextQueryParser;

lazy_static::lazy_static! {
    static ref QUERY_PRATT_PASER : PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::prefix(always) | Op::prefix(exists) | Op::prefix(proba) | Op::prefix(finally) | Op::prefix(globally))
            .op(Op::infix(or, Left))
            .op(Op::infix(and, Left))
            .op(Op::infix(until, Left) | Op::infix(implies, Left))
            .op(Op::prefix(not) | Op::prefix(next))
            .op(
                Op::infix(eq, Left) | Op::infix(ls, Left) | Op::infix(le, Left) |
                Op::infix(gs, Left) | Op::infix(ge, Left) | Op::infix(ne, Left)
            )
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left))
            .op(Op::prefix(minus))
    };
}

#[derive(Debug)]
enum CondOp { CondAnd, CondOr, CondUntil, CondImplies, CondNot, CondNext }
#[derive(Debug)]
enum ExprOp { ExprAdd, ExprSubtract, ExprMultiply, ExprMinus }

#[derive(Debug)]
enum ParsedQuery {
    ParsedExpr(Expr),
    ParsedCond(Condition),
    ParsedUnaryExpr(ExprOp, Box<ParsedQuery>),
    ParsedUnaryCond(CondOp, Box<ParsedQuery>),
    ParsedBinExpr(ExprOp, Box<ParsedQuery>, Box<ParsedQuery>),
    ParsedBinCond(CondOp, Box<ParsedQuery>, Box<ParsedQuery>),
    ParsedBinProp(PropositionType, Box<ParsedQuery>, Box<ParsedQuery>),
    ParsedQuantifier(Quantifier, Box<ParsedQuery>),
    ParsedLogic(StateLogic, Box<ParsedQuery>),
}

impl ParsedQuery {

    pub fn build_query(self) -> QueryParsingResult<Query> {
        match self {
            ParsedQuantifier(q, sub) => {
                let mut next = sub.build_query()?;
                next.quantifier = q;
                Ok(next)
            }
            ParsedLogic(l, sub) => {
                let cond = sub.build_cond()?;
                Ok(Query::new(Quantifier::LTL, l, cond))
            }
            _ => {
                let cond = self.build_cond()?;
                Ok(Query::new(Quantifier::LTL, StateLogic::RawCondition, cond))
            }
        }
    }

    pub fn build_cond(self) -> QueryParsingResult<Condition> {
        match self {
            ParsedCond(c) => Ok(c),
            ParsedBinCond(op, c1, c2) => {
                let cond1 = Box::new(c1.build_cond()?);
                let cond2 = Box::new(c2.build_cond()?);
                match op {
                    CondAnd => Ok(Condition::And(cond1, cond2)),
                    CondOr => Ok(Condition::Or(cond1, cond2)),
                    CondImplies => Ok(Condition::Implies(cond1, cond2)),
                    CondUntil => Ok(Condition::Until(cond1, cond2)),
                    _ => Err(QueryParsingError)
                }
            },
            ParsedUnaryCond(op, c) => {
                let cond = Box::new(c.build_cond()?);
                match op {
                    CondNot => Ok(Condition::Not(cond)),
                    CondNext => Ok(Condition::Next(cond)),
                    _ => Err(QueryParsingError)
                }
            },
            ParsedBinProp(op, e1, e2) => {
                let expr1 = e1.build_expr()?;
                let expr2 = e2.build_expr()?;
                Ok(Condition::Proposition(op, expr1, expr2))
            }
            _ => {
                let expr = self.build_expr()?;
                Ok(Condition::Evaluation(expr))
            }
        }
    }

    pub fn build_expr(self) -> QueryParsingResult<Expr> {
        match self {
            ParsedExpr(e) => Ok(e),
            ParsedUnaryExpr(op, e) => {
                let expr = Box::new(e.build_expr()?);
                match op {
                    ExprMinus => Ok(Expr::Negative(expr)),
                    _ => Err(QueryParsingError)
                }
            },
            ParsedBinExpr(op, e1, e2) => {
                let expr1 = Box::new(e1.build_expr()?);
                let expr2 = Box::new(e2.build_expr()?);
                match op {
                    ExprAdd => Ok(Expr::Plus(expr1, expr2)),
                    ExprSubtract => Ok(Expr::Minus(expr1, expr2)),
                    ExprMultiply => Ok(Expr::Multiply(expr1, expr2)),
                    _ => Err(QueryParsingError)
                }
            }
            _ => Err(QueryParsingError)
        }
    }

}

use ParsedQuery::*;
use CondOp::*;
use ExprOp::*;

fn parse_query_pairs(pairs: Pairs<Rule>) -> ParsedQuery {
    QUERY_PRATT_PASER
        .map_primary(|primary| match primary.as_rule() {
            Rule::ident => ParsedExpr(Expr::Name(Label::from(primary.as_str()))),
            Rule::string_ident => ParsedExpr(Expr::Name(Label::from(primary.as_str()))),
            Rule::int_constant => ParsedExpr(Expr::Constant(primary.as_str().parse::<i32>().unwrap())),
            Rule::r#true => ParsedCond(Condition::True),
            Rule::r#false => ParsedCond(Condition::False),
            Rule::deadlock => ParsedCond(Condition::Deadlock),
            Rule::cond => parse_query_pairs(primary.into_inner()),
            Rule::expr => parse_query_pairs(primary.into_inner()),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule)
        })
        .map_infix(|lhs, op, rhs| {
            let lhs = Box::new(lhs);
            let rhs = Box::new(rhs);
            match op.as_rule() {
                Rule::add => ParsedBinExpr(ExprAdd, lhs, rhs),
                Rule::subtract => ParsedBinExpr(ExprSubtract, lhs, rhs),
                Rule::multiply => ParsedBinExpr(ExprMultiply, lhs, rhs),
                Rule::and => ParsedBinCond(CondAnd, lhs, rhs),
                Rule::or => ParsedBinCond(CondOr, lhs, rhs),
                Rule::until => ParsedBinCond(CondUntil, lhs, rhs),
                Rule::implies => ParsedBinCond(CondImplies, lhs, rhs),
                Rule::eq => ParsedBinProp(PropositionType::EQ, lhs, rhs),
                Rule::ne => ParsedBinProp(PropositionType::NE, lhs, rhs),
                Rule::gs => ParsedBinProp(PropositionType::GS, lhs, rhs),
                Rule::ge => ParsedBinProp(PropositionType::GE, lhs, rhs),
                Rule::ls => ParsedBinProp(PropositionType::LS, lhs, rhs),
                Rule::le => ParsedBinProp(PropositionType::LE, lhs, rhs),
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            }
        })
        .map_prefix(|op, rhs| {
            let rhs = Box::new(rhs);
            match op.as_rule() {
                Rule::not => ParsedUnaryCond(CondNot, rhs),
                Rule::next => ParsedUnaryCond(CondNext, rhs),
                Rule::minus => ParsedUnaryExpr(ExprMinus, rhs),
                Rule::always => ParsedQuantifier(Quantifier::ForAll, rhs),
                Rule::exists => ParsedQuantifier(Quantifier::Exists, rhs),
                Rule::proba => ParsedQuantifier(Quantifier::Probability, rhs),
                Rule::finally => ParsedLogic(StateLogic::Finally, rhs),
                Rule::globally => ParsedLogic(StateLogic::Globally, rhs),
                _ => unreachable!(),
            }
        })
        .parse(pairs)

}

pub fn parse_query(query : String) -> QueryParsingResult<Query> {
    match TextQueryParser::parse(Rule::query, &query) {
        Ok(pairs) => {
            let parsed = parse_query_pairs(pairs);
            //println!("Raw parsed: {:#?}", parsed);
            Ok(parsed.build_query()?)
        }
        Err(e) => {
            eprintln!("Parse failed: {:?}", e);
            Err(QueryParsingError)
        }
    }
}