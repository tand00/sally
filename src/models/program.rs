use super::{expressions::{Condition, Expr}, model_var::ModelVar, ModelState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Program {
    Nop,
    Update(ModelVar, Expr),
    IfElse(Condition, Box<Program>, Box<Program>),
    While(Condition, Box<Program>),
    DoWhile(Condition, Box<Program>),
    For(Box<Program>, Condition, Box<Program>, Box<Program>),
    Block(Vec<Box<Program>>),
    //Definition(ModelVar, VarType)
}

use Program::*;

impl Program {

    pub fn execute(&self, mut state : ModelState) -> ModelState {
        match self {
            Nop => state,
            Update(var, expr) => {
                let res = expr.evaluate(&state);
                var.set(&mut state, res);
                state
            },
            IfElse(c, i, e) => {
                if c.is_true(&state) {
                    i.execute(state)
                } else {
                    e.execute(state)
                }
            },
            While(c, p) => {
                while c.is_true(&state) {
                    state = p.execute(state);
                }
                state
            },
            DoWhile(c, p) => {
                loop {
                    state = p.execute(state);
                    if !c.is_true(&state) {
                        break;
                    }
                }
                state
            },
            For(init, cond, upd, body) => {
                state = init.execute(state);
                while cond.is_true(&state) {
                    state = upd.execute(state);
                    state = body.execute(state);
                }
                state
            },
            Block(statements) => {
                for statement in statements.iter() {
                    state = statement.execute(state);
                }
                state
            }
        }
    }

}

impl Default for Program {
    fn default() -> Self {
        Nop
    }
}