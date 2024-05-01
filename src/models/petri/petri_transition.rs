use std::fmt;

use serde::{Deserialize, Serialize};

use crate::models::time::TimeInterval;
use crate::models::{Edge, Label, ModelState, Node};

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

    #[serde(skip)]
    pub input_edges: Vec<InputEdge>,

    #[serde(skip)]
    pub output_edges: Vec<OutputEdge>,
    
    #[serde(skip)]
    pub index : usize,
}

impl Node for PetriTransition {

    fn get_label(&self) -> Label {
        self.label.clone()
    }

}

impl PetriTransition {

    pub fn new(label : Label, from : Vec<Label>, to : Vec<Label>, interval : TimeInterval) -> Self {
        PetriTransition {
            label, from, to, interval, input_edges: Vec::new(), output_edges: Vec::new(), controllable : true, index : 0
        }
    }

    pub fn new_untimed(label : Label, from : Vec<Label>, to : Vec<Label>) -> Self {
        PetriTransition {
            label, from, to, interval: TimeInterval::full(), input_edges: Vec::new(), output_edges: Vec::new(), controllable : true, index : 0
        }
    }

    pub fn new_uncontrollable(label : Label, from : Vec<Label>, to : Vec<Label>, interval : TimeInterval) -> Self {
        PetriTransition {
            label, from, to, interval, input_edges: Vec::new(), output_edges: Vec::new(), controllable : false, index : 0
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
            if marking.tokens(edge.ptr_node_from().borrow().index) < edge.weight {
                return false
            }
        }
        true
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