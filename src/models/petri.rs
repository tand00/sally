use std::{collections::{HashMap, HashSet}, fmt, rc::{Rc, Weak}};

use crate::computation::virtual_memory::VirtualMemory;

use super::{lbl, model_characteristics::*, model_var::VarType, model_context::ModelContext, new_ptr, time::ClockValue, CompilationResult, ComponentPtr, Edge, Label, Model, ModelMeta, ModelState, Node};

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

#[derive(Clone)]
pub struct PetriNet {
    pub places: Vec<ComponentPtr<PetriPlace>>,
    pub transitions: Vec<ComponentPtr<PetriTransition>>,
    pub places_dic: HashMap<Label, usize>,
    pub transitions_dic: HashMap<Label, usize>,
    pub context : ModelContext
}

impl PetriNet {

    pub fn new(places: Vec<PetriPlace>, transitions : Vec<PetriTransition>) -> Self {
        let mut ctx = ModelContext::new();
        let mut places_dic : HashMap<Label, usize> = HashMap::new();
        let mut transitions_dic : HashMap<Label, usize> = HashMap::new();
        let mut places_ptr : Vec<ComponentPtr<PetriPlace>> = Vec::new();
        let mut transitions_ptr : Vec<ComponentPtr<PetriTransition>> = Vec::new();
        for mut place in places {
            place.index = places_ptr.len();
            places_dic.insert(place.get_label(), place.index);
            places_ptr.push(new_ptr(place));
        }
        for mut transition in transitions {
            transition.index = transitions_ptr.len();
            transitions_dic.insert(transition.get_label(), transition.index);
            transitions_ptr.push(new_ptr(transition));
        }
        let mut petri = PetriNet { 
            places : places_ptr, 
            transitions : transitions_ptr, 
            places_dic, 
            transitions_dic,
            context : ctx
        };
        //petri.compile();
        petri
    }

    pub fn get_place_index(&self, place : &Label) -> usize {
        self.places_dic[place]
    }

    pub fn get_transition_index(&self, transition : &Label) -> usize {
        self.transitions_dic[transition]
    }

    pub fn get_place_label(&self, place : usize) -> Label {
        self.places[place].borrow().get_label()
    }

    pub fn get_transition_label(&self, transition : usize) -> Label {
        self.transitions[transition].borrow().get_label()
    }

    pub fn enabled_transitions(&self, marking : &ModelState) -> Vec<usize> {
        self.transitions.iter().enumerate().filter_map(|(i, transi)| {
            if transi.borrow().is_enabled(marking) { Some(i) }
            else { None }
        }).collect()
    }

    pub fn compute_new_actions(&self, new_state : &mut ModelState, changed_places : &HashSet<usize>) -> (HashSet<usize>, HashSet<usize>) {
        let mut pers = new_state.enabled_clocks();
        let mut newen : HashSet<usize> = HashSet::new();
        for place_index in changed_places {
            let place : &ComponentPtr<PetriPlace> = &self.places[*place_index];
            for transi_weak in place.borrow().get_downstream_transitions().iter() {
                let transition = Weak::upgrade(transi_weak).unwrap();
                let transi_index = transition.borrow().index;
                new_state.disable_clock(transi_index);
                pers.remove(&transi_index);
                if transition.borrow().is_enabled(new_state) {
                    new_state.enable_clock(transi_index, ClockValue::zero());
                    newen.insert(transi_index);
                }
            }
        }
        (newen, pers)
    }

    pub fn fire(&self, mut state : ModelState, action : usize) -> (ModelState, HashSet<usize>, HashSet<usize>) {
        let transi = &self.transitions[action].borrow();
        let mut changed_places : HashSet<usize> = HashSet::new();
        for edge in transi.input_edges.iter() {
            let place_ref = edge.ptr_node_from().borrow();
            let place_var = place_ref.get_var();
            let place_index = place_ref.index;
            state.unmark(place_var, edge.weight);
            changed_places.insert(place_index);
        }
        for edge in transi.output_edges.iter() {
            let place_ref = edge.ptr_node_to().borrow();
            let place_var = place_ref.get_var();
            let place_index = place_ref.index;
            state.mark(place_var, edge.weight);
            changed_places.insert(place_index);
        }
        let (newen, pers) = self.compute_new_actions(&mut state, &changed_places);
        (state, newen, pers)
    }

    fn create_transition_edges(&self, transition : &ComponentPtr<PetriTransition>) {
        let from_labels = transition.borrow().from.clone();
        let to_labels = transition.borrow().to.clone();
        for place_label in from_labels.iter() {
            let place_index = self.places_dic[place_label];
            let place = &self.places[place_index];
            let in_edge = Edge {
                label: lbl(""),
                from: Some(transition.borrow().get_label()), 
                to: Some(place.borrow().get_label()),
                weight : 1,
                ref_from : Some(Rc::downgrade(place)),
                ref_to : Some(Rc::downgrade(transition))
            };
            transition.borrow_mut().input_edges.push(in_edge);
            place.borrow_mut().add_downstream_transition(transition);
        }
        for place_label in to_labels.iter() {
            let place_index = self.places_dic[place_label];
            let place = &self.places[place_index];
            let out_edge = Edge {
                label: lbl(""),
                from: Some(transition.borrow().get_label()), 
                to: Some(place.borrow().get_label()),
                weight : 1,
                ref_from : Some(Rc::downgrade(transition)),
                ref_to : Some(Rc::downgrade(place))
            };
            transition.borrow_mut().output_edges.push(out_edge);
            place.borrow_mut().add_upstream_transition(transition);
        }
    }

    pub fn get_structure(&self) -> impl Serialize {
        let mut places : Vec<PetriPlace> = Vec::new();
        let mut transitions : Vec<PetriTransition> = Vec::new();
        for place_ptr in self.places.iter() {
            let place = place_ptr.borrow().clone();
            places.push(place);
        }
        for transi_ptr in self.transitions.iter() {
            let transi = transi_ptr.borrow().clone();
            transitions.push(transi);
        }
        PetriStructure { places, transitions }
    }

}

impl Model for PetriNet {

    fn next(&self, state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>) {
        let (mut new_state, _, _) = self.fire(state, action);
        let actions: HashSet<usize> = self.available_actions(&new_state);
        if actions.is_empty() && self.available_delay(&new_state).is_zero() {
            new_state.deadlocked = true;
        }
        (Some(new_state), actions)
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<usize> {
        state.clocks.iter().enumerate().filter_map(|(i,c)| {
            if c.is_enabled() && self.transitions[i].borrow().interval.contains(*c) {
                Some(i)
            } else {
                None
            }
        }).collect()
    }

    fn available_delay(&self, state : &ModelState) -> ClockValue {
        let m = state.clocks.iter().enumerate().filter_map(|(i,c)| {
            if c.is_enabled() {
                Some((ClockValue::from(self.transitions[i].borrow().interval.1) - *c).0)
            } else {
                None
            }
        }).reduce(f64::min);
        if m.is_none() {
            ClockValue::zero()
        } else {
            ClockValue(m.unwrap())
        }
    }

    fn init_initial_clocks(&self, mut state : ModelState) -> ModelState {
        state.create_clocks(self.transitions.len());
        for clock in self.enabled_transitions(&state) {
            state.enable_clock(clock, ClockValue::zero());
        }
        state
    }

    fn delay(&self, mut state : ModelState, dt : ClockValue) -> Option<ModelState> {
        state.step(dt);
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
        for place in self.places.iter() {
            let mut place_ref = place.borrow_mut();
            place_ref.clear_upstream_transitions();
            place_ref.clear_downstream_transitions();
            place_ref.compile(context)?;
        }
        for transition in self.transitions.iter() {
            transition.borrow_mut().clear_edges();
            transition.borrow_mut().compile(context)?;
            self.create_transition_edges(transition);
        }
        Ok(())
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