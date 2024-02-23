use crate::models::{Transition, State};
use crate::models::{Model, Label};
use std::fmt;

pub struct Game {
    model: Box<dyn Model>,
    start: Vec<Label>,
    objective: Vec<Label>,
    controllable: Vec<Label>,
}

#[derive(Clone)]
pub enum RunElement {
    RunAction(Box<dyn Transition>),
    RunState(Box<dyn State>),
}
impl RunElement {
    pub fn get_label(&self) -> Label {
        match self {
            RunAction(transi) => transi.get_label(),
            RunState(state) => state.get_label()
        }
    }
}

use RunElement::RunAction;
use RunElement::RunState;

#[derive(Clone)]
pub struct Run {
    elements: Vec<RunElement>,
}
impl Run {
    pub fn empty() -> Run {
        Run { elements: Vec::new() }
    }
    pub fn get_actions_seq(&self) -> Run {
        Run { elements: 
            self.elements.iter().filter_map( |el| match el {
                RunAction(transi) => Some(RunAction(transi.clone())),
                _ => None
            } ).collect() 
        }
    }
    pub fn get_states_seq(&self) -> Run {
        Run { elements: 
            self.elements.iter().filter_map( |el| match el {
                RunState(state) => Some(RunState(state.clone())),
                _ => None
            } ).collect() 
        }
    }
    pub fn steps_count(&self) -> usize {
        self.get_actions_seq().elements.iter().count()
    }
    pub fn pop(&mut self) -> Option<RunElement> {
        self.elements.pop()
    }
    pub fn push(&mut self, el : RunElement) {
        self.elements.push(el);
    }
    pub fn push_state(&mut self, state : Box<dyn State>) {
        self.push(RunState(state));
    }
    pub fn push_action(&mut self, action : Box<dyn Transition>) {
        self.push(RunAction(action));
    }
}
impl fmt::Display for Run {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seq_vec : Vec<String> = self.elements.iter().map(|el| el.get_label().to_string() ).collect();
        let seq_str = seq_vec.join("->");
        write!(f, "{seq_str}")
    }
}