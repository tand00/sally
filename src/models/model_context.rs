use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::computation::virtual_memory::{EvaluationType, VariableDefiner, VirtualMemory};

use super::{action::Action, model_var::{ModelVar, VarType}, Label, Model, ModelState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelContext {
    vars : HashMap<Label, ModelVar>,
    actions : HashMap<Label, Action>,
    definer : VariableDefiner,
    path : Vec<Label>
}

impl ModelContext {

    pub fn new() -> Self {
        ModelContext {
            vars : HashMap::new(),
            actions : HashMap::new(),
            definer : VariableDefiner::new(),
            path : Vec::new()
        }
    }

    pub fn n_vars(&self) -> usize {
        self.vars.len()
    }

    pub fn n_actions(&self) -> usize {
        self.actions.len()
    }

    pub fn make_memory(&self) -> VirtualMemory {
        self.definer.clone().into()
    }

    pub fn get_vars(&self) -> Vec<ModelVar> {
        self.vars.iter().map(|(n,l)| {
            l.clone()
        }).collect()
    }

    pub fn add_var(&mut self, name : Label, var_type : VarType) -> ModelVar {
        let var_name = self.get_local_var_name(name);
        let mut var = ModelVar::name(var_name);
        self.definer.define(&mut var, var_type);
        self.vars.insert(var.name.clone(), var.clone());
        var
    }

    pub fn get_var(&self, name : &Label) -> Option<ModelVar> {
        let var_name = self.get_local_var_name(name.clone());
        if self.vars.contains_key(&var_name) {
            Some(self.vars[&var_name].clone())
        } else {
            None
        }
    }

    pub fn has_var(&self, name : &Label) -> bool {
        let var_name = self.get_local_var_name(name.clone());
        self.vars.contains_key(&var_name)
    }

    pub fn origin(&self) {
        self.path.clear();
    }

    pub fn parent(&mut self) {
        self.path.pop();
    }

    pub fn add_domain(&mut self, domain : Label) {
        self.path.push(domain);
    }

    pub fn has_custom_path(&self) -> bool {
        !self.path.is_empty()
    }

    pub fn get_path(&self) -> Label {
        if self.path.is_empty() {
            return Label::new()
        }
        let mut cwd = self.path[0].clone();
        for domain in self.path.iter().skip(1) {
            cwd += ".";
            cwd += domain.clone();
        }
        cwd
    }

    pub fn get_local_var_name(&self, name : Label) -> Label {
        if self.has_custom_path() {
            self.get_path() + "." + name
        } else {
            name
        }
    }

    fn make_initial_state(&self, model : &impl Model, marking : HashMap<Label, EvaluationType>) -> ModelState {
        let mut state = ModelState::new(self.n_vars(), 0);
        for (k,v) in marking.iter() {
            let var = self.get_var(k);
            if var.is_none() {
                continue;
            }
            let var = var.unwrap();
            state.discrete.set(&var, *v)
        }
        model.init_initial_clocks(state)
    }

    pub fn clear(&mut self) {
        self.vars.clear();
        self.actions.clear();
        self.path.clear();
        self.definer.clear();
    }

}

impl Display for ModelContext {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " [.] ModelContext(\n");
        for (name, var) in self.vars.iter() {
            write!(f, " | {} [{}]\n", name, var.get_address());
        }
        write!(f, ")")
    }

}