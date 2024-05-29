use std::{collections::{HashMap, HashSet}, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::models::{action::Action, model_context::ModelContext, model_var::{ModelVar, VarType}, CompilationResult, Label, Node};
use super::ProbabilisticChoice;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarkovNode {
    pub label : Label,
    pub outputs : HashMap<Label, Vec<(Label, f64)>>,

    #[serde(skip)]
    pub index : usize,
    #[serde(skip)]
    var : ModelVar,
    #[serde(skip)]
    pub actions : HashMap<Action, ProbabilisticChoice<usize>>,
}

impl MarkovNode {

    pub fn new(label : Label) -> MarkovNode {
        MarkovNode {
            label,
            ..Default::default()
        }
    }

    pub fn choice(label : Label, outputs : HashMap<Label, Vec<(Label, f64)>>) -> MarkovNode {
        MarkovNode {
            label,
            outputs,
            ..Default::default()
        }
    }

    pub fn probabilistic(label : Label, outputs : Vec<(Label, f64)>) -> MarkovNode {
        let action = Action::Epsilon;
        MarkovNode {
            label,
            outputs : HashMap::from([ 
                (Label::from(action.to_string()), outputs) 
            ]),
            ..Default::default()
        }
    }

    pub fn get_var(&self) -> &ModelVar {
        &self.var
    }

    pub fn set_var(&mut self, var : ModelVar) {
        self.var = var
    }

    pub fn is_choice(&self) -> bool {
        self.outputs.len() > 1
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        self.set_var(ctx.add_var(self.get_label(), VarType::VarU8));
        if self.is_choice() {
            for action_name in self.outputs.keys() {
                ctx.get_or_add_action(action_name.clone());
            }
        }
        Ok(())
    }

    pub fn has_action(&self, action : &Action) -> bool {
        return self.actions.contains_key(action)
    }

    pub fn available_actions(&self) -> HashSet<Action> {
        self.actions.keys().map(|a| a.clone()).collect()
    }

    pub fn act(&self, action : Action) -> Option<usize> {
        if !self.has_action(&action) {
            return None
        }
        return Some(self.actions[&action].sample().clone())
    }

}

impl Node for MarkovNode {

    fn get_label(&self) -> Label {
        self.label.clone()
    }

}

impl Display for MarkovNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MarkovNode({})", self.get_label())
    }
}