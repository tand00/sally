use std::{collections::HashMap, fmt, process::Output};

use super::{Label, Model, Node};
use crate::computation::ActionSet;
use crate::game::Strategy;

mod petri_place;
mod petri_transition;
mod petri_marking;
mod petri_state;
mod petri_class;

pub use petri_place::PetriPlace;
pub use petri_transition::PetriTransition;
pub use petri_marking::PetriMarking;
pub use petri_state::{PetriState, FiringFunction};
pub use petri_class::PetriClass;

type PetriStrategy = dyn Strategy<Input = PetriState, Output = FiringFunction>;

pub struct PetriNet {
    pub places: Vec<PetriPlace>,
    pub transitions: Vec<PetriTransition>,
    places_dic: HashMap<Label, usize>,
    transitions_dic: HashMap<Label, usize>,
}

impl PetriNet {

    pub fn new(places: Vec<PetriPlace>, transitions : Vec<PetriTransition>) -> Self {
        let mut places_dic : HashMap<Label, usize> = HashMap::new();
        let mut transitions_dic : HashMap<Label, usize> = HashMap::new();
        for (key, place) in places.iter().enumerate() {
            places_dic.insert(place.get_label(), key);
        }
        for (key, transi) in transitions.iter().enumerate() {
            transitions_dic.insert(transi.get_label(), key);
        }
        PetriNet { places, transitions, places_dic, transitions_dic }
    }

    pub fn get_place_index(&self, place : &Label) -> usize {
        self.places_dic[place]
    }

    pub fn get_transition_index(&self, transition : &Label) -> usize {
        self.transitions_dic[transition]
    }

    pub fn get_place_label(&self, place : usize) -> Label {
        self.places[place].get_label()
    }

    pub fn get_transition_label(&self, transition : usize) -> Label {
        self.transitions[transition].get_label()
    }

    pub fn enabled_transitions(&self, marking : &PetriMarking) -> ActionSet {
        ActionSet::new()
    }

    pub fn fire(&self, state : &PetriState, action : usize) -> (PetriState, ActionSet) {
        if !state.firing_function.next_actions().contains(&action) {
            panic!("Transition not fireable !");
        }
        let mut next_state = state.clone();
        next_state.firing_function.step_to_next_action();
        let transi = &self.transitions[action];
        for edge in transi.input_edges.iter() {
            next_state.marking.unmark(edge.node_from(), edge.weight);
        }
        let pers = self.enabled_transitions(&next_state.marking);
        for edge in transi.output_edges.iter() {
            next_state.marking.mark(edge.node_to(), edge.weight);
        }
        let enabled_post = self.enabled_transitions(&next_state.marking);
        let newen = ActionSet::get_newen(&pers, &enabled_post);
        next_state.actions = newen.clone() | pers;
        (next_state, newen)
    }

}

impl Model for PetriNet {
    type State = PetriState;
    type Action = usize;

    fn next(&self, state : &PetriState, action : usize) -> PetriState {
        let (mut next_state, newen) = self.fire(state, action);
        
        next_state
    }

}

// Display implementations ---
impl fmt::Display for PetriNet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let states_str : Vec<String> = self.places.iter().map( |s| s.to_string() ).collect();
        let states_str = states_str.join(";");
        let transition_str : Vec<String> = self.transitions.iter().map( |s| s.to_string() ).collect();
        let transition_str = transition_str.join(";"); 
        let to_print = format!("TimePetriNet_[{}]_[{}]", states_str, transition_str);
        write!(f, "{}", to_print)
    }
}