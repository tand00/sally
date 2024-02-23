use std::fmt;
use std::cmp;
use std::hash;

pub mod time;
//pub mod class_graph;
pub mod petri;
pub mod observation;

/// Abstraction of String to be used in model definitions (transitions and states labels...)
#[derive(Debug, Clone)]
pub struct Label(String);

impl Label {
    pub fn new(lbl : &str) -> Self {
        Label(String::from(lbl))
    }
}
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.0)
    }
}
impl cmp::PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl cmp::Eq for Label {}
impl hash::Hash for Label {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

/// Short label constructor
pub fn lbl(name : &str) -> Label {
    Label::new(name)
}

/// Generic trait that should be implemented by all types of states
pub trait State {
    fn get_label(&self) -> Label;
    fn clone_box(&self) -> Box<dyn State>;
}
impl Clone for Box<dyn State> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Generic trait that should be implemented by all types of transitions
pub trait Transition {
    fn get_label(&self) -> Label;
    fn get_inputs(&self) -> Vec<Label>;
    fn get_outputs(&self) -> Vec<Label>;
    fn clone_box(&self) -> Box<dyn Transition>;
}
impl Clone for Box<dyn Transition> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Generic trait that should be implemented by all types of models
pub trait Model {
    fn get_transitions(&self) -> Vec<Box<&dyn Transition>>;
    fn get_states(&self) -> Vec<Box<&dyn State>>;
    fn get_initial_states(&self) -> Vec<Label>;

    fn check_labels_coherence(&self) -> bool {
        let mut coherent = true;
        let transitions = self.get_transitions();
        let states = self.get_states();
        let state_labels : Vec<Label> = states.iter().map(|s| s.get_label()).collect();
        for transi in transitions {
            let mut transi_states = transi.get_inputs();
            transi_states.extend(transi.get_outputs());
            for input in transi_states.iter() {
                println!("{}", &input);
                if !state_labels.contains(input) {
                    coherent = false;
                }
            }
        }
        for init in self.get_initial_states().iter() {
            if !state_labels.contains(init) {
                coherent = false;
            }
        }
        coherent
    }

    fn check_coherence(&self) -> bool {
        self.check_labels_coherence() // For now only one general method, but may vary depending on the model subclass
    }
}