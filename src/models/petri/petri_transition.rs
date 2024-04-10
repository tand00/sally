use std::collections::HashMap;
use std::fmt;

use crate::models::time::TimeInterval;
use crate::models::{Edge, Label, Node};

use super::PetriMarking;

#[derive(Clone)]
pub struct PetriTransition {
    pub label: Label,
    pub from: Vec<Label>,
    pub to: Vec<Label>,
    pub interval: TimeInterval,
    pub input_edges: Vec<Edge<i32>>,
    pub output_edges: Vec<Edge<i32>>
}

impl Node for PetriTransition {

    fn get_label(&self) -> Label {
        self.label.clone()
    }

}

impl PetriTransition {

    pub fn new(label : Label, from : Vec<Label>, to : Vec<Label>, interval : TimeInterval) -> Self {
        PetriTransition {
            label, from, to, interval, input_edges: Vec::new(), output_edges: Vec::new()
        }
    }

    pub fn new_untimed(label : Label, from : Vec<Label>, to : Vec<Label>) -> Self {
        PetriTransition {
            label, from, to, interval: TimeInterval::full(), input_edges: Vec::new(), output_edges: Vec::new()
        }
    }

    pub fn get_inputs(&self) -> Vec<&Edge<i32>> {
        self.input_edges.iter().collect()
    }

    pub fn get_outputs(&self) -> Vec<&Edge<i32>> {
        self.output_edges.iter().collect()
    }

    pub fn create_edges(&mut self, places_dic : HashMap<Label, usize>, transitions_dic : HashMap<Label, usize>) {
        let this_index = transitions_dic[&self.label];
        for place_label in self.from.iter() {
            let mut edge = Edge::new(place_label.clone(), self.get_label());
            edge.set_node_to(this_index);
            edge.set_node_from(places_dic[place_label]);
            self.input_edges.push(edge);
        }
        for place_label in self.to.iter() {
            let mut edge = Edge::new(self.get_label(), place_label.clone());
            edge.set_node_from(this_index);
            edge.set_node_to(places_dic[place_label]);
            self.output_edges.push(edge);
        }
    }

    pub fn is_enabled(&self, marking : &PetriMarking) -> bool {
        for edge in self.input_edges.iter() {
            if marking.tokens(edge.node_from()) < edge.weight {
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