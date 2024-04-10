use std::{collections::{HashMap, HashSet}, fmt};

use super::{Label, Model, Node};
use crate::computation::ActionSet;

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
        let mut en = ActionSet::new();
        for (i, transi) in self.transitions.iter().enumerate() {
            if transi.is_enabled(marking) {
                en.enable(i);
            }
        }
        en
    }

    pub fn compute_newen_pers(&self, new_marking : &PetriMarking, changed_places : &HashSet<usize>, old_actions : &ActionSet) -> (ActionSet, ActionSet) {
        let mut pers = old_actions.clone();
        let mut newen = ActionSet::new();
        for place_index in changed_places {
            let place : &PetriPlace = &self.places[*place_index];
            for transi_index in place.get_out_transitions() {
                pers.disable(transi_index);
                let transi : &PetriTransition = &self.transitions[transi_index];
                if transi.is_enabled(new_marking) {
                    newen.enable(transi_index);
                }
            }
        }
        (pers, newen)
    }

    pub fn fire(&self, state : &PetriState, action : usize) -> (PetriState, ActionSet) {
        if !state.firing_function.next_actions().contains(&action) {
            panic!("Transition not fireable !");
        }
        let mut next_state = state.clone();
        next_state.firing_function.step_to_next_action();
        let transi = &self.transitions[action];
        let mut changed_places : HashSet<usize> = HashSet::new();
        for edge in transi.input_edges.iter() {
            next_state.marking.unmark(edge.node_from(), edge.weight);
            changed_places.insert(edge.node_from());
        }
        for edge in transi.output_edges.iter() {
            next_state.marking.mark(edge.node_to(), edge.weight);
            changed_places.insert(edge.node_to());
        }
        let (newen, pers) = self.compute_newen_pers(&next_state.marking, &changed_places, &state.actions);
        next_state.new_actions(&newen | &pers);
        (next_state, newen)
    }

}

impl Model for PetriNet {
    type State = PetriState;
    type Action = (FiringFunction, usize);

    fn next(&self, mut state : Self::State, action : Self::Action) -> (Self::State, Vec<usize>) {
        state.firing_function.merge(action.0);
        let (new_state, newen) = self.fire(&state, action.1);
        (new_state, newen.get_actions())
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