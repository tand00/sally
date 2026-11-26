use std::{collections::{HashMap, HashSet}, fmt::Display, iter};

use serde::{Deserialize, Serialize};

use super::model_storage::ModelStorage;

pub type ActionSymbol = u16;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Base(ActionSymbol),
    WithData(ActionSymbol, ModelStorage)
}

impl Action {

    pub fn get_symbol(&self) -> ActionSymbol {
        match self {
            Action::Base(i) 
            | Action::WithData(i, _) => *i
        }
    }

    pub fn base(&self) -> Action {
        Self::Base(self.get_symbol())
    }

    pub fn with_data(&self, data : ModelStorage) -> Action {
        match self {
            Action::Base(i) 
            | Action::WithData(i, _) => Action::WithData(*i, data),
        }
    }

    pub fn extract_data(self) -> Option<(Action, ModelStorage)> {
        match self {
            Self::WithData(i, d) => Some((Self::Base(i), d)),
            _ => None
        }
    }

    pub fn has_data(&self) -> bool {
        match self {
            Self::WithData(_, _) => true,
            _ => false
        }
    }

}

impl Default for Action {
    fn default() -> Self {
        Self::Base(0)
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base(i) => write!(f, "Action({})", i),
            Self::WithData(i, _) => write!(f, "WithData({})", i)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActionPairs(HashSet<Action>, HashSet<Action>);

impl ActionPairs {

    pub fn new() -> ActionPairs {
        Default::default()
    }

    pub fn add_input(&mut self, action : Action) {
        self.0.insert(action);
    }

    pub fn add_output(&mut self, action : Action) {
        self.1.insert(action);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty() || self.1.is_empty()
    }

    pub fn enabled(&self, set : &HashSet<Action>) -> ActionPairs {
        let mut inputs = HashSet::new();
        let mut outputs = HashSet::new();
        for action in set.iter() {
            let base = action.base();
            if self.0.contains(&base) {
                inputs.insert(action.clone());
            }
            if self.1.contains(&base) {
                outputs.insert(action.clone());
            }
        }
        ActionPairs(inputs, outputs)
    }

    pub fn remove_io(&self, mut other : HashSet<Action>) -> HashSet<Action> {
        for input in self.0.iter() {
            if other.contains(input) {
                other.remove(input);
            }
        }
        for output in self.1.iter() {
            if other.contains(output) {
                other.remove(output);
            }
        }
        other
    }

    pub fn choose_pair(&self) -> Option<(Action, Action)> {
        let input = self.0.iter().next();
        let output = self.1.iter().next();
        if input.is_none() || output.is_none() {
            return None;
        }
        Some((input.unwrap().clone(), output.unwrap().clone()))
    }

    pub fn generate_pairs(&self) -> Vec<(Action, Action)> {
        let mut res = Vec::new();
        for input in self.0.iter() {
            for output in self.1.iter() {
                res.push((input.clone(), output.clone()))
            }
        }
        res
    }

}