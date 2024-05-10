use std::{cell::RefCell, fmt, rc::{Rc, Weak}};

use serde::{Serialize, Deserialize};

use crate::{computation::virtual_memory::VirtualMemory, models::{model_var::{ModelVar, VarType}, ComponentPtr, Label, Node}};

use super::PetriTransition;

#[derive(Clone, Serialize, Deserialize)]
pub struct PetriPlace {
    pub name: Label,
    
    #[serde(skip)]
    pub index : usize,

    #[serde(skip)]
    in_transitions : Vec<Weak<RefCell<PetriTransition>>>,

    #[serde(skip)]
    out_transitions: Vec<Weak<RefCell<PetriTransition>>>,

    #[serde(skip)]
    data_variable: Option<ModelVar>
}

impl PetriPlace {

    pub fn new(lbl : Label) -> Self {
        PetriPlace {
            name: lbl,
            index : 0,
            in_transitions : Vec::new(),
            out_transitions : Vec::new(),
            data_variable: None
        }
    }

    pub fn add_upstream_transition(&mut self, transi : &ComponentPtr<PetriTransition>) {
        self.in_transitions.push(Rc::downgrade(transi))
    }

    pub fn clear_upstream_transitions(&mut self) {
        self.in_transitions.clear()
    }

    pub fn get_upstream_transitions(&self) -> &Vec<Weak<RefCell<PetriTransition>>> {
        &self.in_transitions
    }

    pub fn add_downstream_transition(&mut self, transi : &ComponentPtr<PetriTransition>) {
        self.out_transitions.push(Rc::downgrade(transi))
    }

    pub fn clear_downstream_transitions(&mut self) {
        self.out_transitions.clear()
    }

    pub fn get_downstream_transitions(&self) -> &Vec<Weak<RefCell<PetriTransition>>> {
        &self.out_transitions
    }

    pub fn define_var(&mut self, memory : &mut VirtualMemory) {
        let mut data_variable = ModelVar::name(self.get_label());
        memory.define(&mut data_variable, VarType::VarU8);
        self.data_variable = Some(data_variable);
    }

}

impl Node for PetriPlace {

    fn get_label(&self) -> Label {
        self.name.clone()
    }

}

impl fmt::Display for PetriPlace {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Place_{}", self.name)
    }

}