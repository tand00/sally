use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use VarPolicy::*;

use crate::models::{Action, ModelClock, ModelContext, ModelVar, Label};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VarPolicy {
    SumVars,
    MaxVar,
    UnitVar,
}

impl Default for VarPolicy {
    fn default() -> Self {
        SumVars
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ObservationFunction {
    pub vars : HashMap<Label, Label>,
    pub clocks : HashMap<Label, Label>,
    pub actions : HashMap<Label, Label>,
    #[serde(default)]
    pub var_policy : VarPolicy,
}

#[derive(Debug, Default, Clone)]
pub struct ObservationLinks {
    pub vars : HashMap<ModelVar, ModelVar>,
    pub clocks : HashMap<ModelClock, ModelClock>,
    pub actions : HashMap<Action, Action>
}

#[derive(Debug, Default, Clone)]
pub struct ObservationContext {
    pub source : ModelContext,
    pub observed : ModelContext,
    pub links : ObservationLinks
}

impl ObservationFunction {

    pub fn generate_context(&self, source : ModelContext) -> ObservationContext {
        let observed = self.observe_context(&source);
        let links = self.get_links(&source, &observed);
        ObservationContext { source, observed, links }
    }

    pub fn get_links(&self, source : &ModelContext, dest : &ModelContext) -> ObservationLinks {
        let mut links = ObservationLinks::default();
        for var in source.get_vars() {
            if !self.vars.contains_key(&var.name) {
                continue;
            }
            let observed = &self.vars[&var.name];
            links.vars.insert(var.clone(), dest.get_var(observed).unwrap());
        }
        for clock in source.get_clocks() {
            if !self.clocks.contains_key(&clock.name) {
                continue;
            }
            let observed = &self.clocks[&clock.name];
            links.clocks.insert(clock.clone(), dest.get_clock(observed).unwrap());
        }
        for (l,action) in source.get_actions() {
            if !self.actions.contains_key(l) {
                continue;
            }
            let observed = &self.actions[l];
            links.actions.insert(action.clone(), dest.get_action(observed).unwrap());
        }
        links
    }

    pub fn observe_context(&self, source : &ModelContext) -> ModelContext {
        let mut context = ModelContext::new();
        for _ in 0..source.n_models() {
            context.new_model();
        }
        for _ in 0..source.n_storages() {
            context.add_storage();
        }
        for var in source.get_vars() {
            let var_label = var.get_name();
            if !self.vars.contains_key(&var_label) {
                continue;
            }
            let observed_label = self.vars[&var_label].clone();
            context.get_or_add_var(observed_label, var.get_type());
        }
        for clock in source.get_clocks() {
            let clock_label = clock.get_name();
            if !self.clocks.contains_key(&clock_label) {
                continue;
            }
            let observed_label = self.clocks[&clock_label].clone();
            context.get_or_add_clock(observed_label);
        }
        for (action_name, _) in source.get_actions() {
            if !self.actions.contains_key(action_name) {
                continue;
            }
            let observed_label = self.actions[action_name].clone();
            context.get_or_add_action(observed_label);
        }
        context
    }

}
