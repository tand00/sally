use std::fmt;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::models::action::Action;
use crate::models::model_clock::ModelClock;
use crate::models::model_context::ModelContext;
use crate::models::{CompilationError, CompilationResult, Edge, Label, ModelState, Node};
use crate::models::expressions::Condition;

use super::tapn_place::TAPNPlace;
use super::tapn_edge::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TAPNTransition {
    pub label : Label,
    pub from : Vec<Label>,
    pub to : Vec<Label>,
    pub controllable : bool,
    pub guard : Condition,

    #[serde(skip)]
    pub index : usize,

    #[serde(skip)]
    pub input_edges : RwLock<Vec<Arc<InputEdge>>>,

    #[serde(skip)]
    pub output_edges : RwLock<Vec<Arc<OutputEdge>>>,

    #[serde(skip)]
    pub compiled_guard : Condition,

    #[serde(skip)]
    pub action : Action,

    #[serde(skip)]
    pub clock : ModelClock
}

impl Node for TAPNTransition {

    fn get_label(&self) -> Label {
        self.label.clone()
    }

}

impl TAPNTransition {

    pub fn new(label : Label, from : Vec<Label>, to : Vec<Label>) -> Self {
        TAPNTransition {
            label, 
            from, to, 
            controllable : true, 
            guard : Condition::True, 
            ..Default::default()
        }
    }

    pub fn new_uncontrollable(label : Label, from : Vec<Label>, to : Vec<Label>) -> Self {
        TAPNTransition {
            label, 
            from, to, 
            controllable : false, 
            guard : Condition::True,
            ..Default::default()
        }
    }

    pub fn get_inputs(&self) -> Vec<Arc<InputEdge>> {
        self.input_edges.read().unwrap().iter().map(|e| {
            Arc::clone(e)
        }).collect()
    }

    pub fn get_outputs(&self) -> Vec<Arc<OutputEdge>> {
        self.output_edges.read().unwrap().iter().map(|e| {
            Arc::clone(e)
        }).collect()
    }

    pub fn add_input_edge(&self, edge : Edge<TAPNEdgeData, TAPNPlace, TAPNTransition>) {
        self.input_edges.write().unwrap().push(Arc::new(edge))
    }

    pub fn add_output_edge(&self, edge : Edge<TAPNEdgeData, TAPNTransition, TAPNPlace>) {
        self.output_edges.write().unwrap().push(Arc::new(edge))
    }

    pub fn is_enabled(&self, marking : &ModelState) -> bool {
        for edge in self.input_edges.read().unwrap().iter() {
            if !edge.has_source() {
                panic!("Every transition edge should have a source");
            }
            if marking.tokens(edge.get_node_from().get_var()) < edge.data().weight {
                return false
            }
        }
        self.compiled_guard.is_true(marking)
    }

    pub fn is_fireable(&self, state : &ModelState) -> bool {
        todo!()
    }

    pub fn clear_edges(&self) {
        self.input_edges.write().unwrap().clear();
        self.output_edges.write().unwrap().clear();
    }

    pub fn inertia(&self) -> i32 {
        let mut res : i32 = 0;
        for e in self.input_edges.read().unwrap().iter() {
            res -= e.data().weight;
        }
        for e in self.output_edges.read().unwrap().iter() {
            res += e.data().weight;
        }
        res
    }

    pub fn is_conservative(self) -> bool {
        return self.inertia() == 0
    }

    pub fn set_clock(&mut self, clock : ModelClock) {
        self.clock = clock;
    }

    pub fn get_clock(&self) -> &ModelClock {
        &self.clock
    }

    pub fn set_action(&mut self, action : Action) {
        self.action = action
    }

    pub fn get_action(&self) -> Action {
        self.action.clone()
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        let res = self.guard.apply_to(ctx);
        match res {
            Ok(c) => {
                self.compiled_guard = c
            },
            Err(_) => return Err(CompilationError)
        };
        self.set_action(ctx.add_action(self.get_label()));
        self.set_clock(ctx.add_clock(self.get_label()));
        Ok(())
    }

}

impl fmt::Display for TAPNTransition {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO : Maybe add from / to in the display text ?
        let from_str : Vec<String> = self.from.iter().map( |lbl| lbl.to_string() ).collect();
        let to_str : Vec<String> = self.to.iter().map( |lbl| lbl.to_string() ).collect();
        let from_str = from_str.join(",");
        let to_str = to_str.join(",");
        let to_print = format!("Transition_{}_[{}]->[{}]", self.label, from_str, to_str);
        write!(f, "{}", to_print)
    }
    
}

impl Clone for TAPNTransition {

    fn clone(&self) -> Self {
        TAPNTransition {
            label: self.label.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
            controllable : self.controllable.clone(),
            guard : self.guard.clone(),
            index : self.index,
            ..Default::default()
        }
    }

}