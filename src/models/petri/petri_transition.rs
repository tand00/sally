use std::fmt;

use serde::{Deserialize, Serialize};

use crate::models::action::Action;
use crate::models::model_clock::ModelClock;
use crate::models::model_context::ModelContext;
use crate::models::time::TimeInterval;
use crate::models::{CompilationError, CompilationResult, Edge, Label, ModelState, Node};
use crate::models::expressions::Condition;

use super::PetriPlace;

pub type InputEdge = Edge<i32, PetriPlace, PetriTransition>;
pub type OutputEdge = Edge<i32, PetriTransition, PetriPlace>;

#[derive(Clone, Serialize, Deserialize)]
pub struct PetriTransition {
    pub label: Label,
    pub from: Vec<Label>,
    pub to: Vec<Label>,
    pub interval: TimeInterval,
    pub controllable : bool,
    pub guard : Condition,

    #[serde(skip)]
    pub input_edges: Vec<InputEdge>,

    #[serde(skip)]
    pub output_edges: Vec<OutputEdge>,
    
    #[serde(skip)]
    pub index : usize,

    #[serde(skip)]
    pub compiled_guard : Condition,

    #[serde(skip)]
    pub action : Action,

    #[serde(skip)]
    pub clock : Option<ModelClock>
}

impl Node for PetriTransition {

    fn get_label(&self) -> Label {
        self.label.clone()
    }

}

impl PetriTransition {

    pub fn new(label : Label, from : Vec<Label>, to : Vec<Label>, interval : TimeInterval) -> Self {
        PetriTransition {
            label, 
            from, to, 
            interval, 
            input_edges: Vec::new(), output_edges: Vec::new(), 
            controllable : true, 
            index : 0, 
            guard : Condition::True, compiled_guard : Condition::True,
            action : Action::Epsilon, clock : None
        }
    }

    pub fn new_untimed(label : Label, from : Vec<Label>, to : Vec<Label>) -> Self {
        PetriTransition {
            label, 
            from, to, 
            interval: TimeInterval::full(), 
            input_edges: Vec::new(), output_edges: Vec::new(), 
            controllable : true, 
            index : 0,
            guard : Condition::True, compiled_guard : Condition::True,
            action : Action::Epsilon, clock : None
        }
    }

    pub fn new_uncontrollable(label : Label, from : Vec<Label>, to : Vec<Label>, interval : TimeInterval) -> Self {
        PetriTransition {
            label, 
            from, to, 
            interval, 
            input_edges: Vec::new(), output_edges: Vec::new(), 
            controllable : false, 
            index : 0,
            guard : Condition::True, compiled_guard : Condition::True,
            action : Action::Epsilon, clock : None
        }
    }

    pub fn get_inputs(&self) -> Vec<&InputEdge> {
        self.input_edges.iter().collect()
    }

    pub fn get_outputs(&self) -> Vec<&OutputEdge> {
        self.output_edges.iter().collect()
    }

    pub fn is_enabled(&self, marking : &ModelState) -> bool {
        for edge in self.input_edges.iter() {
            if !edge.has_source() {
                panic!("Every transition edge should have a source");
            }
            if marking.tokens(&edge.ptr_node_from().borrow().get_var()) < edge.weight {
                return false
            }
        }
        self.compiled_guard.is_true(marking)
    }

    pub fn is_fireable(&self, state : &ModelState) -> bool {
        false
    }

    pub fn clear_edges(&mut self) {
        self.input_edges.clear();
        self.output_edges.clear();
    }

    pub fn inertia(&self) -> i32 {
        let mut res : i32 = 0;
        for e in self.input_edges.iter() {
            res -= e.weight;
        }
        for e in self.output_edges.iter() {
            res += e.weight;
        }
        res
    }

    pub fn is_conservative(self) -> bool {
        return self.inertia() == 0
    }

    pub fn get_clock(&self) -> &ModelClock {
        match &self.clock {
            None => panic!("Transition clock is not set !"),
            Some(i) => i
        }
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        let res = self.guard.apply_to(ctx);
        match res {
            Ok(c) => self.compiled_guard = c,
            Err(_) => return Err(CompilationError)
        };
        self.action = ctx.add_action(self.get_label());
        self.clock = Some(ctx.add_clock(self.get_label()));
        Ok(())
    }

}

impl fmt::Display for PetriTransition {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO : Maybe add from / to in the display text ?
        let from_str : Vec<String> = self.from.iter().map( |lbl| lbl.to_string() ).collect();
        let to_str : Vec<String> = self.to.iter().map( |lbl| lbl.to_string() ).collect();
        let from_str = from_str.join(",");
        let to_str = to_str.join(",");
        let to_print = format!("Transition_{}_{}_[{}]->[{}]", self.label, self.interval, from_str, to_str);
        write!(f, "{}", to_print)
    }
    
}