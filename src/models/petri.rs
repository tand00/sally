use super::{Label, Model, State, Transition};
use super::time::TimeInterval;
use std::fmt;

#[derive(Clone)]
pub struct PetriState(pub Label);
impl State for PetriState {
    fn get_label(&self) -> Label {
        self.0.clone()
    }
    fn clone_box(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct PetriTransition {
    pub label: Label,
    pub from: Vec<Label>,
    pub to: Vec<Label>,
    pub interval: TimeInterval,
}
impl Transition for PetriTransition {
    fn get_label(&self) -> Label {
        self.label.clone()
    }
    fn get_inputs(&self) -> Vec<Label> {
        self.from.iter().map(|l| l.clone()).collect()
    }
    fn get_outputs(&self) -> Vec<Label> {
        self.to.iter().map(|l| l.clone()).collect()
    }
    fn clone_box(&self) -> Box<dyn Transition> {
        Box::new(self.clone())
    }
}

pub struct PetriNet {
    pub states: Vec<PetriState>,
    pub transitions: Vec<PetriTransition>,
    pub initial_states: Vec<Label>,
}
impl Model for PetriNet {
    fn get_states(&self) -> Vec<Box<&dyn State>> {
        self.states.iter().map(|state| Box::new(state as _)).collect()
    }
    fn get_transitions(&self) -> Vec<Box<&dyn Transition>> {
        self.transitions.iter().map(|transi| Box::new(transi as _)).collect()
    }
    fn get_initial_states(&self) -> Vec<Label> {
        self.initial_states.iter().map(|lbl| lbl.clone() ).collect()
    }
}

// Display implementations ---

impl fmt::Display for PetriState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "State_{}", self.0)
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
impl fmt::Display for PetriNet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let states_str : Vec<String> = self.states.iter().map( |s| s.to_string() ).collect();
        let states_str = states_str.join(";");
        let transition_str : Vec<String> = self.transitions.iter().map( |s| s.to_string() ).collect();
        let transition_str = transition_str.join(";"); 
        let to_print = format!("TimePetriNet_[{}]_[{}]", states_str, transition_str);
        write!(f, "{}", to_print)
    }
}