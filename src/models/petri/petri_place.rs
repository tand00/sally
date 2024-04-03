use std::fmt;
use crate::models::{Label, Node};

#[derive(Clone)]
pub struct PetriPlace {
    pub name: Label,
    out_transitions: Vec<usize>
}

impl PetriPlace {

    pub fn new(lbl : Label) -> Self {
        PetriPlace {
            name: lbl,
            out_transitions: Vec::new()
        }
    } 

    pub fn add_out_transition(&mut self, action : usize) {
        self.out_transitions.push(action)
    }

    pub fn clear_out_transitions(&mut self) {
        self.out_transitions.clear()
    }

}

impl Node for PetriPlace {

    fn get_label(&self) -> Label {
        self.name.clone()
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }

}

impl fmt::Display for PetriPlace {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Place_{}", self.name)
    }

}
