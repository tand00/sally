use std::fmt;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::computation::intervals::Convex;
use crate::models::action::Action;
use crate::models::model_clock::ModelClock;
use crate::models::model_context::ModelContext;
use crate::models::time::TimeInterval;
use crate::models::{CompilationError, CompilationResult, Edge, Label, ModelState, Node};
use crate::models::expressions::Condition;

use super::PetriPlace;

pub type InputEdge = Edge<i32, PetriPlace, PetriTransition>;
pub type OutputEdge = Edge<i32, PetriTransition, PetriPlace>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PetriTransition {
    pub label: Label,
    pub from: Vec<(Label, i32)>,
    pub to: Vec<(Label, i32)>,
    pub interval: TimeInterval,
    pub controllable : bool,
    pub guard : Condition,

    #[serde(skip)]
    pub index : usize,

    #[serde(skip)]
    pub input_edges: OnceLock<Vec<InputEdge>>,

    #[serde(skip)]
    pub output_edges: OnceLock<Vec<OutputEdge>>,

    #[serde(skip)]
    pub compiled_guard : Condition,

    #[serde(skip)]
    pub action : Action,

    #[serde(skip)]
    pub clock : ModelClock
}

impl Node for PetriTransition {

    fn get_label(&self) -> Label {
        self.label.clone()
    }

}

impl PetriTransition {

    pub fn new(label : Label, from : Vec<(Label, i32)>, to : Vec<(Label, i32)>, interval : TimeInterval) -> Self {
        PetriTransition {
            label,
            from, to,
            interval,
            controllable : true,
            guard : Condition::True,
            ..Default::default()
        }
    }

    pub fn safe(label : Label, from : Vec<Label>, to : Vec<Label>, interval : TimeInterval) -> Self {
        PetriTransition {
            label,
            from : from.into_iter().map(|l| (l,1)).collect::<Vec<(Label, i32)>>(), 
            to : to.into_iter().map(|l| (l,1)).collect::<Vec<(Label, i32)>>(),
            interval,
            controllable : true,
            guard : Condition::True,
            ..Default::default()
        }
    }

    pub fn new_untimed(label : Label, from : Vec<(Label, i32)>, to : Vec<(Label, i32)>) -> Self {
        PetriTransition {
            label,
            from, to,
            controllable : true,
            interval : TimeInterval::full(),
            guard : Condition::True,
            ..Default::default()
        }
    }

    pub fn new_uncontrollable(label : Label, from : Vec<(Label, i32)>, to : Vec<(Label, i32)>, interval : TimeInterval) -> Self {
        PetriTransition {
            label,
            from, to,
            interval,
            controllable : false,
            guard : Condition::True,
            ..Default::default()
        }
    }

    #[inline]
    pub fn get_inputs(&self) -> &Vec<InputEdge> {
        self.input_edges.get().unwrap()
    }

    #[inline]
    pub fn get_outputs(&self) -> &Vec<OutputEdge> {
        self.output_edges.get().unwrap()
    }

    pub fn is_enabled(&self, marking : &ModelState) -> bool {
        for edge in self.get_inputs().iter() {
            if !edge.has_source() {
                panic!("Every transition edge should have a source");
            }
            if edge.get_node_from().tokens(marking) < edge.weight {
                return false
            }
        }
        self.compiled_guard.is_true(marking)
    }

    pub fn is_fireable(&self, state : &ModelState) -> bool {
        let clockvalue = state.get_clock_value(self.get_clock());
        if clockvalue.is_disabled() {
            return false;
        }
        self.interval.contains(&clockvalue)
    }

    pub fn clear_edges(&mut self) {
        self.input_edges = OnceLock::new();
        self.output_edges = OnceLock::new();
    }

    pub fn inertia(&self) -> i32 {
        let mut res : i32 = 0;
        for e in self.get_inputs().iter() {
            res -= e.weight;
        }
        for e in self.get_outputs().iter() {
            res += e.weight;
        }
        res
    }

    #[inline]
    pub fn is_conservative(self) -> bool {
        return self.inertia() == 0
    }

    pub fn set_clock(&mut self, clock : ModelClock) {
        self.clock = clock;
    }

    #[inline]
    pub fn get_clock(&self) -> &ModelClock {
        &self.clock
    }

    pub fn set_action(&mut self, action : Action) {
        self.action = action
    }

    #[inline]
    pub fn get_action(&self) -> Action {
        self.action.clone()
    }

    pub fn untimed(&self) -> Self {
        PetriTransition {
            label: self.label.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
            interval: TimeInterval::full(),
            controllable : self.controllable.clone(),
            guard : self.guard.clone(),
            index : self.index,
            ..Default::default()
        }
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

impl fmt::Display for PetriTransition {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO : Maybe add from / to in the display text ?
        let from_str : Vec<String> = self.from.iter().map( |lbl| lbl.0.to_string() ).collect();
        let to_str : Vec<String> = self.to.iter().map( |lbl| lbl.0.to_string() ).collect();
        let from_str = from_str.join(",");
        let to_str = to_str.join(",");
        let to_print = format!("Transition_{}_{}_[{}]->[{}]", self.label, self.interval, from_str, to_str);
        write!(f, "{}", to_print)
    }

}

impl Clone for PetriTransition {

    fn clone(&self) -> Self {
        PetriTransition {
            label: self.label.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
            interval: self.interval.clone(),
            controllable : self.controllable.clone(),
            guard : self.guard.clone(),
            index : self.index,
            ..Default::default()
        }
    }

}
