use std::{collections::HashMap, fmt::Display};

use crate::computation::virtual_memory::{EvaluationType, VariableDefiner, VirtualMemory};

use super::{action::Action, model_clock::ModelClock, model_storage::ModelStorage, model_var::{ModelVar, VarType}, Label, Model, ModelState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelContext {
    n_models : usize,
    n_storages : usize,
    vars : HashMap<Label, ModelVar>,
    actions : HashMap<Label, Action>,
    clocks : HashMap<Label, ModelClock>,
    //io_actions : HashMap<Label, usize>,
    definer : VariableDefiner,
    path : Vec<Label>,
}

impl ModelContext {

    pub fn new() -> Self {
        ModelContext {
            n_models : 0,
            n_storages : 0,
            vars : HashMap::new(),
            actions : HashMap::new(),
            clocks : HashMap::new(),
            //io_actions : HashMap::new(),
            definer : VariableDefiner::new(),
            path : Vec::new(),
        }
    }

    pub fn n_models(&self) -> usize {
        self.n_models
    }

    pub fn new_model(&mut self) -> usize {
        let id = self.n_models;
        self.n_models += 1;
        id
    }

    pub fn add_storage(&mut self) -> usize {
        let id = self.n_storages;
        self.n_storages += 1;
        id
    }

    pub fn n_storages(&self) -> usize {
        self.n_storages
    }

    pub fn n_vars(&self) -> usize {
        self.vars.len()
    }

    pub fn n_actions(&self) -> usize {
        self.actions.len()
    }

    pub fn n_clocks(&self) -> usize {
        self.clocks.len()
    }

    pub fn make_memory(&self) -> VirtualMemory {
        self.definer.clone().into()
    }

    pub fn get_vars(&self) -> Vec<ModelVar> {
        self.vars.iter().map(|(_,l)| {
            l.clone()
        }).collect()
    }

    pub fn add_var(&mut self, name : Label, var_type : VarType) -> ModelVar {
        let var_name = self.get_local_name(name);
        let mut var = ModelVar::name(var_name);
        self.definer.define(&mut var, var_type);
        self.vars.insert(var.name.clone(), var.clone());
        var
    }

    pub fn get_var(&self, name : &Label) -> Option<ModelVar> {
        let mut scope = self.path.clone();
        while scope.len() > 0 {
            let mut cwd = Label::new();
            for domain in scope.iter() {
                cwd += domain.clone() + ".";
            }
            let var_name = cwd + name;
            if self.vars.contains_key(&var_name) {
                return Some(self.vars[&var_name].clone())
            }
            scope.pop();
        }
        if self.vars.contains_key(&name) {
            return Some(self.vars[&name].clone())
        }
        None
    }

    pub fn has_var(&self, name : &Label) -> bool {
        let var_name = self.get_local_name(name.clone());
        self.vars.contains_key(&var_name)
    }

    pub fn add_action(&mut self, name : Label) -> Action {
        let id = self.n_actions();
        let action_name = self.get_local_name(name.clone());
        let action = Action::Internal(id);
        self.actions.insert(action_name, action);
        action
    }

    pub fn get_action(&self, name : &Label) -> Option<Action> {
        let action_name = self.get_local_name(name.clone());
        if self.actions.contains_key(&action_name) {
            Some(self.actions[&action_name].clone())
        } else {
            None
        }
    }

    pub fn has_action(&self, name : &Label) -> bool {
        let local_name = self.get_local_name(name.clone());
        self.actions.contains_key(&local_name)
    }

    pub fn add_clock(&mut self, name : Label) -> ModelClock {
        let clock_name = self.get_local_name(name);
        let mut clock = ModelClock::name(clock_name);
        clock.index = self.n_clocks();
        self.clocks.insert(clock.name.clone(), clock.clone());
        clock
    }

    pub fn get_clock(&self, name : &Label) -> Option<ModelClock> {
        let local_name = self.get_local_name(name.clone());
        if self.clocks.contains_key(&local_name) {
            Some(self.clocks[&local_name].clone())
        } else {
            None
        }
    }

    pub fn has_clock(&self, name : &Label) -> bool {
        let local_name = self.get_local_name(name.clone());
        self.clocks.contains_key(&local_name)
    }

    pub fn get_or_add_var(&mut self, name : Label, var_type : VarType)  -> ModelVar {
        let var = self.get_var(&name);
        match var {
            Some(v) => v,
            None => self.add_var(name, var_type)
        }
    }

    pub fn get_or_add_action(&mut self, name : Label)  -> Action {
        let var = self.get_action(&name);
        match var {
            Some(v) => v,
            None => self.add_action(name)
        }
    }

    pub fn get_or_add_clock(&mut self, name : Label)  -> ModelClock {
        let var = self.get_clock(&name);
        match var {
            Some(v) => v,
            None => self.add_clock(name)
        }
    }

    
    pub fn get_local_vars(&self) -> Vec<ModelVar> {
        let domain = self.get_path();
        self.vars.iter().filter_map(|(l,x)| {
            if l.has_domain(&domain) {
                Some(x.clone())
            } else {
                None
            }
        }).collect()
    }

    pub fn get_local_actions(&self) -> Vec<Action> {
        let domain = self.get_path();
        self.actions.iter().filter_map(|(l,x)| {
            if l.has_domain(&domain) {
                Some(x.clone())
            } else {
                None
            }
        }).collect()
    }

    pub fn get_local_clocks(&self) -> Vec<ModelClock> {
        let domain = self.get_path();
        self.clocks.iter().filter_map(|(l,x)| {
            if l.has_domain(&domain) {
                Some(x.clone())
            } else {
                None
            }
        }).collect()
    }

    pub fn origin(&mut self) {
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

    pub fn get_local_name(&self, name : Label) -> Label {
        if self.has_custom_path() {
            self.get_path() + "." + name
        } else {
            name
        }
    }

    pub fn make_initial_state(&self, model : &impl Model, marking : HashMap<Label, EvaluationType>) -> ModelState {
        let mut state = ModelState::new(self.n_vars(), self.n_clocks());
        state.storages.resize(self.n_storages(), ModelStorage::EmptyStorage);
        for (k,v) in marking.iter() {
            let var = self.get_var(k);
            if var.is_none() {
                continue;
            }
            let var = var.unwrap();
            state.discrete.set(&var, *v)
        }
        state = model.init_initial_clocks(state);
        model.init_initial_storage(state)
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
        write!(f, " [.] ModelContext\n | - Models : {}\n", self.n_models())?;
        write!(f, " | - Vars :\n")?;
        for (name, var) in self.vars.iter() {
            write!(f, " | {} [{}]\n", name, var.get_address())?;
        }
        write!(f, " | - Clocks :\n")?;
        for (name, clock) in self.clocks.iter() {
            write!(f, " | {} [{}]\n", name, clock.get_index())?;
        }
        write!(f, " | - Actions :\n")?;
        for (name, action) in self.actions.iter() {
            write!(f, " | {} [{}]\n", name, action)?;
        }
        write!(f, " | - Storages : [{}]\n", self.n_storages())
    }

}