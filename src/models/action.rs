use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

use super::model_storage::ModelStorage;

// Action enum :
// Epsilon : No label nor ID, used for internal invisible transitions

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "_")]
    Epsilon,
    Internal(usize),
    Sync(usize, Box<Action>, Box<Action>),
    WithData(usize, ModelStorage)
}

impl Action {

    pub fn get_id(&self) -> usize {
        match self {
            Self::Epsilon => usize::MAX,
            Self::Internal(i) => *i,
            Self::Sync(i, _, _) => *i,
            Self::WithData(i, _) => *i
        }
    }

    pub fn base(&self) -> Action {
        match self {
            Self::Epsilon => Self::Epsilon,
            Self::Internal(_) => self.clone(),
            _ => Self::Internal(self.get_id())
        }
    }

}

impl Default for Action {
    fn default() -> Self {
        Self::Epsilon
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Epsilon => write!(f, "_"),
            Self::Internal(i) => write!(f, "Action({})", i),
            Self::Sync(id, i, j) => write!(f, "Sync({},{},{})", id, i, j),
            Self::WithData(i, d) => write!(f, "WithData({})", i)
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
        let inputs : HashSet<Action> = self.0.intersection(set).map(|a| a.clone()).collect();
        let outputs : HashSet<Action> = self.1.intersection(set).map(|a| a.clone()).collect();
        ActionPairs(inputs, outputs)
    }

    pub fn remove_io(&self, other : HashSet<Action>) -> HashSet<Action> {
        let other : HashSet<Action> = other.difference(&self.0).map(|x| x.clone()).collect();
        other.difference(&self.1).map(|x| x.clone()).collect()
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