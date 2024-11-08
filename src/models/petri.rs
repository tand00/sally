use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use crate::{models::{class_graph::StateClassGenerator, digraph::search_strategy::BreadthFirst}, verification::{smc::RandomRunIterator, Verifiable, VerificationBound}};

use super::{
    action::Action, lbl, model_characteristics::*, model_context::ModelContext, time::{ClockValue, RealTimeBound},
    CompilationResult, Edge, Label, Model, ModelMaker, ModelMeta, ModelState, Node,
};

mod petri_place;
mod petri_transition;

use num_traits::Zero;
pub use petri_place::PetriPlace;
pub use petri_transition::PetriTransition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PetriStructure {
    pub places: Vec<PetriPlace>,
    pub transitions: Vec<PetriTransition>,
}

#[derive(Debug, Clone)]
pub struct PetriNet {
    pub id: usize,
    pub places: Vec<Arc<PetriPlace>>,
    pub transitions: Vec<Arc<PetriTransition>>,
    pub places_dic: HashMap<Label, usize>,
    pub transitions_dic: HashMap<Label, usize>,
    pub actions_dic: HashMap<Action, usize>,
    pub timed: bool,
}

impl PetriNet {

    pub fn new(places: Vec<PetriPlace>, transitions: Vec<PetriTransition>) -> Self {
        let mut places_ptr: Vec<Arc<PetriPlace>> = Vec::new();
        let mut transitions_ptr: Vec<Arc<PetriTransition>> = Vec::new();
        for place in places {
            places_ptr.push(Arc::new(place));
        }
        for transition in transitions {
            transitions_ptr.push(Arc::new(transition));
        }
        let petri = PetriNet {
            id: usize::MAX,
            places: places_ptr,
            transitions: transitions_ptr,
            places_dic: HashMap::new(),
            transitions_dic: HashMap::new(),
            actions_dic: HashMap::new(),
            timed: true,
        };
        petri
    }

    pub fn get_place(&self, place: &Label) -> Arc<PetriPlace> {
        Arc::clone(&self.places[self.places_dic[place]])
    }

    pub fn get_transition(&self, transition: &Label) -> Arc<PetriTransition> {
        Arc::clone(&self.transitions[self.transitions_dic[transition]])
    }

    pub fn enabled_transitions(&self, marking: &ModelState) -> Vec<Arc<PetriTransition>> {
        self.transitions
            .iter()
            .filter_map(|transi| {
                if transi.is_enabled(marking) {
                    Some(transi.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn disable_transitions(&self, new_state : &mut ModelState, changed_places: &HashSet<usize>) {
        let mut transition_seen = vec![false; self.transitions.len()];
        for place_index in changed_places {
            let place: &Arc<PetriPlace> = &self.places[*place_index];
            for transition in place.get_downstream_transitions().iter() {
                let transi_index = transition.index;
                if transition_seen[transi_index] {
                    continue;
                }
                transition_seen[transi_index] = true;
                if !new_state.is_enabled(transition.get_clock()) {
                    continue;
                }
                let clock = transition.get_clock();
                if !transition.is_enabled(new_state) {
                    new_state.disable_clock(clock);
                }
            }
        }
    }

    pub fn compute_new_actions(&self, new_state: &mut ModelState, changed_places: &HashSet<usize>) 
        -> (HashSet<usize>, HashSet<usize>) 
    {
        let mut pers = new_state.enabled_clocks();
        let mut newen: HashSet<usize> = HashSet::new();
        let mut transition_seen = vec![false; self.transitions.len()];
        for place_index in changed_places {
            let place: &Arc<PetriPlace> = &self.places[*place_index];
            for transition in place.get_downstream_transitions().iter() {
                let transi_index = transition.index;
                if transition_seen[transi_index] {
                    continue;
                }
                transition_seen[transi_index] = true;
                let clock = transition.get_clock();
                if transition.is_enabled(new_state) {
                    if !new_state.is_enabled(clock) {
                        new_state.enable_clock(clock, ClockValue::zero());
                        newen.insert(transi_index);
                    }
                } else {
                    new_state.disable_clock(clock);
                    pers.remove(&transi_index);
                }
            }
        }
        (newen, pers)
    }

    pub fn fire(&self, mut state: ModelState, transi: usize) 
        -> (ModelState, HashSet<usize>, HashSet<usize>) 
    {
        let transi = &self.transitions[transi];
        let mut changed_places: HashSet<usize> = HashSet::new();
        state.disable_clock(transi.get_clock());
        for edge in transi.get_inputs().iter() {
            let place_ptr = edge.get_node_from();
            let place_var = place_ptr.get_var();
            let place_index = place_ptr.index;
            state.unmark(place_var, edge.weight);
            changed_places.insert(place_index);
        }
        self.disable_transitions(&mut state, &changed_places);
        for edge in transi.get_outputs().iter() {
            let place_ptr = edge.get_node_to();
            let place_var = place_ptr.get_var();
            let place_index = place_ptr.index;
            state.mark(place_var, edge.weight);
            changed_places.insert(place_index);
        }
        let (mut newen, pers) = self.compute_new_actions(&mut state, &changed_places);
        if !transi.has_preset() {
            state.enable_clock(transi.get_clock(), ClockValue::zero());
            newen.insert(transi.index);
        }
        (state, newen, pers)
    }

    fn create_transition_edges(
        &self, transition: &Arc<PetriTransition>, 
        places_down : &mut Vec<Vec<Arc<PetriTransition>>>, places_up : &mut Vec<Vec<Arc<PetriTransition>>>
    ) {
        let from_labels = transition.from.clone();
        let to_labels = transition.to.clone();
        let guard_vars = transition.compiled_guard.get_objects().vars;
        let mut input_edges = Vec::new();
        let mut output_edges = Vec::new();
        for place_from in from_labels.iter() {
            let place_index = self.places_dic[&place_from.0];
            let place = &self.places[place_index];
            let in_edge = Edge::data_edge(place, transition, place_from.1);
            input_edges.push(in_edge);
            places_down[place_index].push(Arc::clone(transition));
        }
        transition.input_edges.set(input_edges).unwrap();
        for place in self.places.iter() {
            let place_var = place.get_var();
            if !guard_vars.contains(place_var) {
                continue;
            }
            places_down[place.index].push(Arc::clone(transition));
        }
        for place_to in to_labels.iter() {
            let place_index = self.places_dic[&place_to.0];
            let place = &self.places[place_index];
            let out_edge = Edge::data_edge(transition, place, place_to.1);
            output_edges.push(out_edge);
            places_up[place_index].push(Arc::clone(transition));
        }
        transition.output_edges.set(output_edges).unwrap();
    }

    pub fn get_structure(&self) -> PetriStructure {
        let mut places: Vec<PetriPlace> = Vec::new();
        let mut transitions: Vec<PetriTransition> = Vec::new();
        for place_ptr in self.places.iter() {
            let place = PetriPlace::clone(place_ptr);
            places.push(place);
        }
        for transi_ptr in self.transitions.iter() {
            let transi = PetriTransition::clone(transi_ptr);
            transitions.push(transi);
        }
        PetriStructure {
            places,
            transitions,
        }
    }

    pub fn get_transition_action(&self, transi_index: usize) -> Action {
        self.transitions[transi_index].get_action()
    }

    pub fn untimed(&self) -> Self {
        let places = self.places.iter().map(|p| PetriPlace::clone(p)).collect();
        let transitions = self.transitions.iter().map(|t| t.untimed()).collect();
        let mut untimed_net = PetriNet::new(places, transitions);
        untimed_net.timed = false;
        untimed_net
    }

    pub fn is_safe(&self, k : i32, initial : &ModelState) -> bool {
        for class in StateClassGenerator::classes(BreadthFirst::new(), self, initial) {
            for place in self.places.iter() {
                if class.evaluate_var(place.get_var()) > k {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn is_1safe(&self, initial : &ModelState) -> bool {
        self.is_safe(1, initial)
    }

}

impl Model for PetriNet {

    fn next(&self, state: ModelState, action: Action) -> Option<ModelState> {
        let transi = self.actions_dic[&action];
        let (mut new_state, newen, pers) = self.fire(state, transi);
        if newen.len() == 0 && pers.len() == 0 {
            new_state.deadlocked = true;
            return Some(new_state);
        }
        Some(new_state)
    }

    fn available_actions(&self, state: &ModelState) -> HashSet<Action> {
        let mut res = HashSet::new();
        for transition in self.transitions.iter() {
            if transition.is_fireable(state) {
                res.insert(transition.get_action());
            }
        }
        res
    }

    fn available_delay(&self, state: &ModelState) -> RealTimeBound {
        if !self.is_timed() {
            return RealTimeBound::Infinite;
        }
        let m = self.transitions.iter()
            .filter_map(|t| {
                let c = state.get_clock_value(t.get_clock());
                if c.is_enabled() {
                    Some(t.interval.1.real() - c)
                } else {
                    None
                }
            })
            .reduce(|a,b| if a < b { a } else { b });
        match m {
            None => RealTimeBound::zero(),
            Some(c) => c,
        }
    }

    fn init_initial_clocks(&self, mut state: ModelState) -> ModelState {
        for transition in self.enabled_transitions(&state) {
            state.enable_clock(transition.get_clock(), ClockValue::zero());
        }
        state
    }

    fn delay(&self, mut state: ModelState, dt: ClockValue) -> Option<ModelState> {
        let clocks = self.transitions.iter().map(|t| t.get_clock());
        state.step_clocks(clocks, dt);
        Some(state)
    }

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name: lbl("TPN"),
            description: String::from(
                "Time Petri net, every transition is associated with a firing interval.",
            ),
            characteristics: TIMED | CONTROLLABLE,
        }
    }

    #[inline]
    fn is_timed(&self) -> bool {
        self.timed
    }

    #[inline]
    fn is_stochastic(&self) -> bool {
        false
    }

    fn compile(&mut self, context: &mut ModelContext) -> CompilationResult<()> {
        self.id = context.new_model();
        self.places_dic.clear();
        self.transitions_dic.clear();
        self.actions_dic.clear();
        let mut compiled_places = Vec::new();
        let mut compiled_transitions = Vec::new();
        let mut places_up = vec![Vec::new() ; self.places.len()];
        let mut places_down = vec![Vec::new() ; self.places.len()];
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
            self.create_transition_edges(&compiled_transition, &mut places_down, &mut places_up);
            compiled_transitions.push(compiled_transition);
        }
        for place in self.places.iter() {
            let index = place.index;
            place.out_transitions.set(places_down[index].clone()).unwrap();
            place.in_transitions.set(places_up[index].clone()).unwrap();
        }
        self.transitions = compiled_transitions;
        Ok(())
    }

    fn random_run<'a>(&'a self, initial : &'a ModelState, bound : VerificationBound) 
        -> Box<dyn Iterator<Item = (std::rc::Rc<ModelState>, ClockValue, Option<Action>)> + 'a> 
    {
        Box::new(RandomRunIterator::generate(self, initial, bound))
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
    pub structure: PetriStructure,
}

impl ModelMaker<PetriNet> for PetriMaker {
    fn create_maker(model: PetriNet) -> Self {
        Self::from(model)
    }

    fn make(&self) -> (PetriNet, ModelContext) {
        let mut new_net = PetriNet::from(self.structure.clone());
        let ctx = new_net.singleton().unwrap();
        (new_net, ctx)
    }
}

impl From<PetriStructure> for PetriMaker {
    fn from(value: PetriStructure) -> Self {
        PetriMaker { structure: value }
    }
}

impl From<PetriNet> for PetriMaker {
    fn from(value: PetriNet) -> Self {
        PetriMaker {
            structure: value.get_structure(),
        }
    }
}

impl From<&PetriNet> for PetriStructure {
    fn from(value: &PetriNet) -> Self {
        value.get_structure()
    }
}
