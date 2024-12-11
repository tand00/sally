use std::{collections::{HashMap, HashSet}, fmt::Display, iter};

use serde::{Deserialize, Serialize};

use crate::computation::BitSet;

use super::{model_storage::ModelStorage, UNMAPPED_ID};

// Action enum :
// Epsilon : No label nor ID, used for internal invisible transitions

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "_")]
    Epsilon,
    Base(usize),
    Sync(usize, Box<Action>, Box<Action>),
    WithData(usize, ModelStorage)
}

impl Action {

    pub fn get_id(&self) -> usize {
        match self {
            Self::Epsilon => UNMAPPED_ID,
            Self::Base(i) => *i,
            Self::Sync(i, _, _) => *i,
            Self::WithData(i, _) => *i
        }
    }

    pub fn is_epsilon(&self) -> bool {
        *self == Self::Epsilon
    }

    pub fn base(&self) -> Action {
        match self {
            Self::Epsilon => Self::Epsilon,
            Self::Base(_) => self.clone(),
            _ => Self::Base(self.get_id())
        }
    }

    pub fn with_data(&self, data : ModelStorage) -> Action {
        Self::WithData(self.get_id(), data)
    }

    pub fn extract_data(self) -> Option<(Action, ModelStorage)> {
        match self {
            Self::WithData(i, d) => Some((Self::Base(i), d)),
            _ => None
        }
    }

    pub fn sync(&self, a : Action, b : Action) -> Action {
        Self::Sync(self.get_id(), Box::new(a), Box::new(b))
    }

    pub fn has_data(&self) -> bool {
        match self {
            Self::WithData(_, _) => true,
            _ => false
        }
    }

    pub fn is_sync(&self) -> bool {
        match self {
            Self::Sync(_, _, _) => true,
            _ => false
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
            Self::Base(i) => write!(f, "Action({})", i),
            Self::Sync(id, i, j) => write!(f, "Sync({},{},{})", id, i, j),
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

#[derive(Debug, Clone)]
pub struct ActionSet {
    pub enabled : BitSet,
    pub has_spe : BitSet,
    pub specializations : HashMap<usize, HashSet<Action>>
}

impl ActionSet {

    pub fn is_enabled(&self, action : &Action) -> bool {
        if action.is_epsilon() {
            return false;
        }
        self.enabled.is_enabled(&action.get_id())
    }

    pub fn add_action(&mut self, action : Action) {
        match action {
            Action::Epsilon => (),
            Action::Base(i) => self.enabled.enable(&i),
            a => {
                let id = a.get_id();
                self.enabled.enable(&id);
                self.has_spe.enable(&id);
                if self.specializations.contains_key(&id) {
                    self.specializations.get_mut(&id).unwrap().insert(a);
                } else {
                    let mut set = HashSet::new();
                    set.insert(a);
                    self.specializations.insert(id, set);
                }
            },
        }
    }

    pub fn disable_action(&mut self, action : &Action) {
        if !self.is_enabled(action) { return }
        let id = action.get_id();
        self.enabled.disable(&id);
        if self.has_spe.is_enabled(&id) {
            self.has_spe.disable(&id);
            self.specializations.remove(&id);
        }
    }

    pub fn unspecialize(&mut self, action : &Action) {
        if !self.has_specialization(action) { return }
        let id = action.get_id();
        self.has_spe.disable(&id);
        self.specializations.remove(&id);
    }

    pub fn remove_specialization(&mut self, action : &Action) {
        if !self.has_specialization(action) { return }
        let id = action.get_id();
        self.specializations.get_mut(&id).unwrap().remove(action);
        if self.specializations[&id].is_empty() {
            self.has_spe.disable(&id);
            self.specializations.remove(&id);
        }
    }

    pub fn has_specialization(&self, action : &Action) -> bool {
        if action.is_epsilon() {
            return false;
        }
        self.has_spe.is_enabled(&action.get_id())
    }

    pub fn base_actions<'a>(&'a self) -> impl Iterator<Item = Action> + 'a {
        self.enabled.get_bits().map(|bit| Action::Base(bit))
    }

    pub fn get_specializations<'a>(&'a self, action : &Action) -> Box<dyn Iterator<Item = &'a Action> + 'a> {
        if !self.has_specialization(action) {
            return Box::new(iter::empty());
        }
        Box::new(self.specializations[&action.get_id()].iter())
    }

    pub fn enabledness_intersection(self, other : Self) -> Self {
        let (mut enabled, has_spe, mut spe) = (self.enabled, self.has_spe, self.specializations);
        let (o_en, o_has_spe, mut o_spe) = (other.enabled, other.has_spe, other.specializations);
        enabled &= o_en;
        let has_spe = enabled.clone() & (has_spe | o_has_spe);
        let mut specializations = HashMap::new();
        for bit in has_spe.get_bits() {
            let mut spe1 = spe.remove(&bit).unwrap();
            let spe2 = o_spe.remove(&bit).unwrap();
            spe1.extend(spe2);
            specializations.insert(bit, spe1);
        }
        ActionSet {
            enabled, has_spe, specializations
        }
    }

}
