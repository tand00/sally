use std::{collections::{HashMap, HashSet}, fmt, sync::{Arc, Weak}};

use crate::verification::Verifiable;

use super::{action::Action, lbl, model_characteristics::*, model_context::ModelContext, time::ClockValue, CompilationResult, Edge, Label, Model, ModelMaker, ModelMeta, ModelState, Node};

mod petri_place;
mod petri_transition;

use num_traits::Zero;
pub use petri_place::PetriPlace;
pub use petri_transition::PetriTransition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PetriStructure {
    pub places : Vec<PetriPlace>,
    pub transitions : Vec<PetriTransition>
}

#[derive(Debug, Clone)]
pub struct PetriNet {
    pub id : usize,
    pub places: Vec<Arc<PetriPlace>>,
    pub transitions: Vec<Arc<PetriTransition>>,
    pub places_dic: HashMap<Label, usize>,
    pub transitions_dic: HashMap<Label, usize>,
    pub actions_dic : HashMap<Action, usize>,
}

impl PetriNet {

    pub fn new(places: Vec<PetriPlace>, transitions : Vec<PetriTransition>) -> Self {
        let mut places_ptr : Vec<Arc<PetriPlace>> = Vec::new();
        let mut transitions_ptr : Vec<Arc<PetriTransition>> = Vec::new();
        for place in places {
            places_ptr.push(Arc::new(place));
        }
        for transition in transitions {
            transitions_ptr.push(Arc::new(transition));
        }
        let petri = PetriNet { 
            id : usize::MAX,
            places : places_ptr, 
            transitions : transitions_ptr, 
            places_dic : HashMap::new(), 
            transitions_dic : HashMap::new(),
            actions_dic : HashMap::new()
        };
        petri
    }

    pub fn get_place(&self, place : &Label) -> Arc<PetriPlace> {
        Arc::clone(&self.places[self.places_dic[place]])
    }

    pub fn get_transition(&self, transition : &Label) -> Arc<PetriTransition> {
        Arc::clone(&self.transitions[self.transitions_dic[transition]])
    }

    pub fn enabled_transitions(&self, marking : &ModelState) -> Vec<Arc<PetriTransition>> {
        self.transitions.iter().filter_map(|transi| {
            if transi.is_enabled(marking) { Some(transi.clone()) }
            else { None }
        }).collect()
    }

    pub fn compute_new_actions(&self, new_state : &mut ModelState, changed_places : &HashSet<usize>) -> (HashSet<usize>, HashSet<usize>) {
        let mut pers = new_state.enabled_clocks();
        let mut newen : HashSet<usize> = HashSet::new();
        let mut transition_seen = vec![false ; self.transitions.len()];
        for place_index in changed_places {
            let place : &Arc<PetriPlace> = &self.places[*place_index];
            for transition_weak in place.out_transitions.read().unwrap().iter() {
                let transition = Weak::upgrade(transition_weak).unwrap();
                let transi_index = transition.index;
                if transition_seen[transi_index] {
                    continue;
                }
                transition_seen[transi_index] = true;
                let clock = transition.get_clock();
                new_state.disable_clock(clock);
                pers.remove(&transi_index);
                if transition.is_enabled(new_state) {
                    new_state.enable_clock(clock, ClockValue::zero());
                    newen.insert(transi_index);
                }
            }
        }
        (newen, pers)
    }

    pub fn fire(&self, mut state : ModelState, transi : usize) -> (ModelState, HashSet<usize>, HashSet<usize>) {
        let transi = &self.transitions[transi];
        let mut changed_places : HashSet<usize> = HashSet::new();
        for edge in transi.input_edges.read().unwrap().iter() {
            let place_ptr = edge.get_node_from();
            let place_var = place_ptr.get_var();
            let place_index = place_ptr.index;
            state.unmark(place_var, edge.weight);
            changed_places.insert(place_index);
        }
        for edge in transi.output_edges.read().unwrap().iter() {
            let place_ptr = edge.get_node_to();
            let place_var = place_ptr.get_var();
            let place_index = place_ptr.index;
            state.mark(place_var, edge.weight);
            changed_places.insert(place_index);
        }
        let (newen, pers) = self.compute_new_actions(&mut state, &changed_places);
        (state, newen, pers)
    }

    fn create_transition_edges(&self, transition : &Arc<PetriTransition>) {
        let from_labels = transition.from.clone();
        let to_labels = transition.to.clone();
        let guard_vars = transition.compiled_guard.get_objects().vars;
        for place_label in from_labels.iter() {
            let place_index = self.places_dic[place_label];
            let place = &self.places[place_index];
            let in_edge = Edge::data_edge(place, transition, 1);
            transition.add_input_edge(in_edge);
            place.add_downstream_transition(transition);
        }
        for place in self.places.iter() {
            let place_var = place.get_var();
            if !guard_vars.contains(place_var) {
                continue
            }
            place.add_downstream_transition(transition);
        }
        for place_label in to_labels.iter() {
            let place_index = self.places_dic[place_label];
            let place = &self.places[place_index];
            let out_edge = Edge::data_edge(transition, place, 1);
            transition.add_output_edge(out_edge);
            place.add_upstream_transition(transition);
        }
    }

    pub fn get_structure(&self) -> PetriStructure {
        let mut places : Vec<PetriPlace> = Vec::new();
        let mut transitions : Vec<PetriTransition> = Vec::new();
        for place_ptr in self.places.iter() {
            let place = PetriPlace::clone(place_ptr);
            places.push(place);
        }
        for transi_ptr in self.transitions.iter() {
            let transi = PetriTransition::clone(transi_ptr);
            transitions.push(transi);
        }
        PetriStructure { places, transitions }
    }

    pub fn get_transition_action(&self, transi_index : usize) -> Action {
        self.transitions[transi_index].get_action()
    }

}

impl Model for PetriNet {

    fn next(&self, state : ModelState, action : Action) -> Option<(ModelState, HashSet<Action>)> {
        let transi = self.actions_dic[&action];
        let (mut new_state, _, _) = self.fire(state, transi);
        let actions: HashSet<Action> = self.available_actions(&new_state);
        if actions.is_empty() && self.available_delay(&new_state).is_zero() {
            new_state.deadlocked = true;
        }
        Some((new_state, actions))
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<Action> {
        let mut res = HashSet::new();
        for transition in self.transitions.iter() {
            if transition.is_fireable(state) {
                res.insert(transition.get_action());
            }
        }
        res
    }

    fn available_delay(&self, state : &ModelState) -> ClockValue {
        let m = self.transitions.iter().filter_map(|t| {
            let c = state.get_clock_value(t.get_clock());
            if c.is_enabled() {
                Some(ClockValue::from(t.interval.1) - c)
            } else {
                None
            }
        }).reduce(ClockValue::min);
        match m {
            None => ClockValue::zero(),
            Some(c) => c
        }
    }

    fn init_initial_clocks(&self, mut state : ModelState) -> ModelState {
        for transition in self.enabled_transitions(&state) {
            state.enable_clock(transition.get_clock(), ClockValue::zero());
        }
        state
    }

    fn delay(&self, mut state : ModelState, dt : ClockValue) -> Option<ModelState> {
        let clocks = self.transitions.iter().map(|t| t.get_clock());
        state.step_clocks(clocks, dt);
        Some(state)
    }

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name : lbl("TPN"),
            description : String::from("Time Petri net, every transition is associated with a firing interval."),
            characteristics : TIMED | CONTROLLABLE,
        }
    }

    fn is_timed(&self) -> bool {
        true
    }

    fn is_stochastic(&self) -> bool {
        false
    }

    fn compile(&mut self, context : &mut ModelContext) -> CompilationResult<()> {
        self.id = context.new_model();
        self.places_dic.clear();
        self.transitions_dic.clear();
        self.actions_dic.clear();
        let mut compiled_places = Vec::new();
        let mut compiled_transitions = Vec::new();
        for (i, place) in self.places.iter().enumerate() {
            let mut compiled_place = PetriPlace::clone(place);
            compiled_place.index = i;
            self.places_dic.insert(compiled_place.get_label(), compiled_place.index);
            compiled_place.compile(context)?;
            compiled_places.push(Arc::new(compiled_place));
        }
        self.places = compiled_places;
        for (i, transition) in self.transitions.iter().enumerate() {
            let mut compiled_transition = PetriTransition::clone(transition);
            compiled_transition.index = i;
            self.transitions_dic.insert(compiled_transition.get_label(), compiled_transition.index);
            compiled_transition.compile(context)?;
            self.actions_dic.insert(compiled_transition.get_action(), compiled_transition.index);
            let compiled_transition = Arc::new(compiled_transition);
            self.create_transition_edges(&compiled_transition);
            compiled_transitions.push(compiled_transition);
        }
        self.transitions = compiled_transitions;
        Ok(())
    }

    fn get_id(&self) -> usize {
        self.id
    }

}

// Display implementations ---
impl fmt::Display for PetriNet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimePetriNet")
    }
}

impl From<PetriStructure> for PetriNet {
    fn from(value: PetriStructure) -> Self {
        PetriNet::new(value.places, value.transitions)
    }
}

pub struct PetriMaker {
    pub structure : PetriStructure
}

impl ModelMaker<PetriNet> for PetriMaker {

    fn create_maker(model : PetriNet) -> Self {
        Self::from(model)
    }

    fn make(&self) -> (PetriNet, ModelContext) {
        let mut new_net = PetriNet::from(self.structure.clone());
        let ctx = new_net.singleton();
        (new_net, ctx)
    }

}

impl From<PetriStructure> for PetriMaker {

    fn from(value : PetriStructure) -> Self {
        PetriMaker {
            structure : value
        }
    }

}

impl From<PetriNet> for PetriMaker {

    fn from(value: PetriNet) -> Self {
        PetriMaker {
            structure : value.get_structure()
        }
    }

}