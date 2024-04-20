use std::{collections::{HashMap, HashSet}, fmt};

use super::{lbl, model_characteristics::*, time::ClockValue, translation, Label, Model, ModelMeta, ModelState, Node};

mod petri_place;
mod petri_transition;

use num_traits::Zero;
pub use petri_place::PetriPlace;
pub use petri_transition::PetriTransition;

#[derive(Clone)]
pub struct PetriNet {
    pub places: Vec<PetriPlace>,
    pub transitions: Vec<PetriTransition>,
    places_dic: HashMap<Label, usize>,
    transitions_dic: HashMap<Label, usize>,
}

impl PetriNet {

    pub fn new(places: Vec<PetriPlace>, transitions : Vec<PetriTransition>) -> Self {
        let mut transitions = transitions;
        let mut places_dic : HashMap<Label, usize> = HashMap::new();
        let mut transitions_dic : HashMap<Label, usize> = HashMap::new();
        for (key, place) in places.iter().enumerate() {
            places_dic.insert(place.get_label(), key);
        }
        for (key, transi) in transitions.iter_mut().enumerate() {
            transitions_dic.insert(transi.get_label(), key);
            transi.create_edges(key, &places_dic);
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

    pub fn enabled_transitions(&self, marking : &ModelState) -> Vec<usize> {
        self.transitions.iter().enumerate().filter_map(|(i, transi)| {
            if transi.is_enabled(marking) { Some(i) }
            else { None }
        }).collect()
    }

    pub fn compute_new_actions(&self, new_state : &mut ModelState, changed_places : &HashSet<usize>) -> (HashSet<usize>, HashSet<usize>) {
        let mut pers = new_state.enabled_clocks();
        let mut newen : HashSet<usize> = HashSet::new();
        for place_index in changed_places {
            let place : &PetriPlace = &self.places[*place_index];
            for transi_index in place.get_out_transitions() {
                new_state.disable_clock(transi_index);
                pers.remove(&transi_index);
                let transi : &PetriTransition = &self.transitions[transi_index];
                if transi.is_enabled(new_state) {
                    newen.insert(transi_index);
                }
            }
        }
        (pers, newen)
    }

    pub fn fire(&self, mut state : ModelState, action : usize) -> (ModelState, HashSet<usize>, HashSet<usize>) {
        let transi = &self.transitions[action];
        let mut changed_places : HashSet<usize> = HashSet::new();
        for edge in transi.input_edges.iter() {
            state.unmark(edge.node_from(), edge.weight);
            changed_places.insert(edge.node_from());
        }
        for edge in transi.output_edges.iter() {
            state.mark(edge.node_to(), edge.weight);
            changed_places.insert(edge.node_to());
        }
        let (newen, pers) = self.compute_new_actions(&mut state, &changed_places);
        (state, newen, pers)
    }

}

impl Model for PetriNet {

    fn next(&self, state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>) {
        let (mut new_state, _, _) = self.fire(state, action);
        let actions: HashSet<usize> = self.actions_available(&new_state);
        if actions.is_empty() && self.available_delay(&new_state).is_zero() {
            new_state.deadlocked = true;
        }
        (Some(new_state), actions)
    }

    fn actions_available(&self, state : &ModelState) -> HashSet<usize> {
        state.clocks.iter().enumerate().filter_map(|(i,c)| {
            if c.is_enabled() && self.transitions[i].interval.contains(*c) {
                Some(i)
            } else {
                None
            }
        }).collect()
    }

    fn available_delay(&self, state : &ModelState) -> ClockValue {
        let m = state.clocks.iter().enumerate().map(|(i,c)| {
            (ClockValue::from(self.transitions[i].interval.1) - *c).0
        }).reduce(f64::min);
        if m.is_none() {
            ClockValue::zero()
        } else {
            ClockValue(m.unwrap())
        }
    }

    fn delay(&self, mut state : ModelState, dt : ClockValue) -> Option<ModelState> {
        state.step(dt);
        Some(state)
    }

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name : lbl("TimePetriNet"),
            description : String::from("Time Petri net, every transition is associated with a firing interval."),
            characteristics : TIMED | CONTROLLABLE,
        }
    }

    fn n_vars(&self) -> usize {
        self.places.len()
    }

    fn n_clocks(&self) -> usize {
        self.transitions.len()
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