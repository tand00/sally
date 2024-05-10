use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Label;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamingContext {
    vars : HashMap<Label, usize>,
    actions : HashMap<Label, usize>,
    sub_contexts : HashMap<Label, Box<NamingContext>>,
    n_vars : usize,
    n_actions : usize,
}

impl NamingContext {

    pub fn new() -> Self {
        NamingContext {
            vars: HashMap::new(),
            actions: HashMap::new(),
            sub_contexts: HashMap::new(),
            n_vars: 0,
            n_actions: 0
        }
    }
    
    pub fn add_sub_context(&mut self, name : Label, sub : NamingContext) {
        self.n_vars += sub.n_vars();
        self.n_actions += sub.n_actions();
        self.sub_contexts.insert(name, Box::new(sub));
    }

    pub fn add_var(&mut self, name : Label, value : usize) {
        self.vars.insert(name, value);
        self.n_vars += 1;
    }

    pub fn add_action(&mut self, name : Label, value : usize) {
        self.actions.insert(name, value);
        self.n_actions += 1;
    }

    pub fn n_vars(&self) -> usize {
        self.n_vars
    }

    pub fn n_actions(&self) -> usize {
        self.n_actions
    }

}