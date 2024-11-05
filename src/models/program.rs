use super::{action::Action, expressions::{Condition, Expr}, model_var::ModelVar, ModelState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Program {
    Nop,
    Update(ModelVar, Expr),
    IfElse(Condition, Box<Program>, Box<Program>),
    Switch(Vec<(Condition, Program)>),
    While(Condition, Box<Program>),
    DoWhile(Condition, Box<Program>),
    For(Box<Program>, Condition, Box<Program>, Box<Program>),
    Block(Vec<Program>),
    // Listener is a special instruction that listens for an incoming Action and executes the associated code block
    Listener(Vec<(Action, Program)>),
    // Definition is used to define variables, useful to manage scopes
    Definition(ModelVar)
}

use Program::*;

impl Program {

    pub fn execute(&self, mut state : ModelState) -> ModelState {
        match self {
            Update(var, expr) => {
                let res = expr.evaluate(&state);
                //var.set(&mut state, res);
                state.set_var(var, res);
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
            Switch(conds) => {
                for (cond, prog) in conds.iter() {
                    if cond.is_true(&state) {
                        state = prog.execute(state);
                        break;
                    }
                }
                state
            },
            Listener(_) => { // Listener cannot be instantaneously executed
                state.deadlocked = true;
                state
            },
            Definition(_) => state,
            Nop => state,
        }
    }

    pub fn has_listeners(&self) -> bool {
        match self {
            Nop => false,
            Update(_, _) => false,
            IfElse(_, program1, program2) => 
                program1.has_listeners() || program2.has_listeners(),
            Switch(vec) => 
                vec.iter().any(|x| x.1.has_listeners()),
            While(_, program) => program.has_listeners(),
            DoWhile(_, program) => program.has_listeners(),
            For(program, _, program1, program2) => 
                program.has_listeners() || program1.has_listeners() || program2.has_listeners(),
            Block(vec) => 
                vec.iter().any(Program::has_listeners),
            Listener(_) => true,
            Definition(_) => false,
        }
    }

}

impl Default for Program {
    fn default() -> Self {
        Nop
    }
}