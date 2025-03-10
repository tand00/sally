use std::{collections::{HashMap, HashSet}, rc::Rc, sync::Arc, usize};

use num_traits::Zero;
use serde::{Deserialize, Serialize};
use tapn_place::TAPNPlace;
use tapn_run_generator::TAPNRunGenerator;
use tapn_token::*;
use tapn_transition::TAPNTransition;

use crate::verification::VerificationBound;

use super::{action::Action, lbl, model_context::ModelContext, model_storage::ModelStorage, time::{ClockValue, RealTimeBound}, CompilationResult, Edge, Label, Model, ModelMeta, ModelState, Node, CONTROLLABLE, TIMED, UNMAPPED_ID};

pub mod tapn_place;
pub mod tapn_edge;
pub mod tapn_transition;
pub mod tapn_token;
pub mod tapn_run_generator;

#[derive(Clone, Serialize, Deserialize)]
pub struct TAPNStructure {
    pub places: Vec<TAPNPlace>,
    pub transitions: Vec<TAPNTransition>,
}

pub struct TAPN {
    pub id : usize,
    pub tokens_storage : usize,
    pub places : Vec<Arc<TAPNPlace>>,
    pub transitions : Vec<Arc<TAPNTransition>>,
    pub places_dic : HashMap<Label, usize>,
    pub actions_dic : HashMap<Action, usize>
}

impl TAPN {

    pub fn new(places : Vec<TAPNPlace>, transitions : Vec<TAPNTransition>) -> Self {
        TAPN {
            id: UNMAPPED_ID,
            tokens_storage: UNMAPPED_ID,
            places: places.into_iter().map(Arc::new).collect(),
            transitions: transitions.into_iter().map(Arc::new).collect(),
            places_dic: HashMap::new(),
            actions_dic: HashMap::new()
        }
    }

    pub fn transition_for_action(&self, action : &Action) -> &Arc<TAPNTransition> {
        &self.transitions[self.actions_dic[action]]
    }

    pub fn fire(&self, mut state : ModelState, transi : usize, in_tokens : TAPNPlaceList) -> ModelState {
        let mut places_tokens = TAPNPlaceListWriter::from(state.mut_storage(&self.tokens_storage));
        let transi = &(self.transitions[transi]);
        let mut vars_updates = Vec::new();
        for edge in transi.get_inputs().iter() {
            let place = edge.get_node_from();
            vars_updates.push((place.clone(), -edge.data().weight));
            let mut state_tokens = places_tokens.place(place.index);
            let input_tokens = &in_tokens.places[place.index];
            state_tokens.remove_set(input_tokens);
        }
        for edge in transi.get_transports().iter() {
            let place = edge.get_node_from();
            let target = edge.get_node_to();
            vars_updates.push((place.clone(), -edge.data().weight));
            vars_updates.push((target.clone(), edge.data().weight));
            let mut state_tokens = places_tokens.place(place.index);
            let input_tokens = &in_tokens.places[place.index];
            state_tokens.remove_set(input_tokens);
            let mut target_tokens = places_tokens.place(target.index);
            for token in input_tokens.iter() {
                target_tokens.insert(token.clone());
            }
        }
        for edge in transi.get_outputs().iter() {
            let target = edge.get_node_to();
            vars_updates.push((target.clone(), edge.weight));
            let mut target_tokens = places_tokens.place(target.index);
            target_tokens.insert(TAPNToken { count: edge.weight, age: ClockValue::zero() });
        }
        for (place, delta) in vars_updates {
            state.mark(place.get_var(), delta);
        }
        state
    }

    pub fn create_transition_edges(
        &self, transition : &Arc<TAPNTransition>,
        places_down : &mut Vec<Vec<Arc<TAPNTransition>>>, places_up : &mut Vec<Vec<Arc<TAPNTransition>>>
    ) {
        let mut input_edges = Vec::new();
        for (place, data) in transition.from.iter() {
            let place_index = self.places_dic[place];
            let place = &self.places[place_index];
            let edge = Edge::data_edge(place, transition, *data);
            input_edges.push(edge);
            places_down[place_index].push(Arc::clone(transition));
        }
        transition.input_edges.set(input_edges).unwrap();
        let mut output_edges = Vec::new();
        for (place, weight) in transition.to.iter() {
            let place_index = self.places_dic[place];
            let place = &self.places[place_index];
            let edge = Edge::data_edge(transition, place, *weight);
            output_edges.push(edge);
            places_up[place_index].push(Arc::clone(transition));
        }
        transition.output_edges.set(output_edges).unwrap();
        let mut inhibs = Vec::new();
        for (place, data) in transition.inhibitors.iter() {
            let place_index = self.places_dic[place];
            let place = &self.places[place_index];
            let edge = Edge::data_edge(place, transition, *data);
            inhibs.push(edge);
            places_down[place_index].push(Arc::clone(transition));
        }
        transition.inhibitor_edges.set(inhibs).unwrap();
        let mut transports = Vec::new();
        for (source, target, data) in transition.transports.iter() {
            let source_index = self.places_dic[source];
            let target_index = self.places_dic[target];
            let source_place = &self.places[source_index];
            let target_place = &self.places[target_index];
            let edge = Edge::data_edge(source_place, target_place, *data);
            transports.push(edge);
            places_down[source_index].push(Arc::clone(transition));
            places_up[target_index].push(Arc::clone(transition));
        }
        transition.transport_edges.set(transports).unwrap();
    }

    pub fn get_structure(&self) -> TAPNStructure {
        let mut places: Vec<TAPNPlace> = Vec::new();
        let mut transitions: Vec<TAPNTransition> = Vec::new();
        for place_ptr in self.places.iter() {
            let place = TAPNPlace::clone(place_ptr);
            places.push(place);
        }
        for transi_ptr in self.transitions.iter() {
            let transi = TAPNTransition::clone(transi_ptr);
            transitions.push(transi);
        }
        TAPNStructure {
            places,
            transitions,
        }
    }

}

impl Model for TAPN {

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name: lbl("TAPN"),
            description: String::from("Timed-Arcs Petri net"),
            characteristics: TIMED | CONTROLLABLE
        }
    }

    fn next(&self, state : ModelState, action : Action) -> Option<ModelState> {
        let Some((action, data)) = action.extract_data() else {
            return None;
        };
        let in_tokens = TAPNPlaceList::from(data);
        let transi = self.actions_dic[&action];
        let next_state = self.fire(state, transi, in_tokens);
        Some(next_state)
    }

    fn delay(&self, mut state : ModelState, dt : ClockValue) -> Option<ModelState> {
        let storage = state.mut_storage(&self.tokens_storage);
        let mut place_list = TAPNPlaceListWriter::from(storage);
        place_list.delta(dt);
        if self.available_delay(&state) == RealTimeBound::MinusInfinite {
            return None;
        }
        Some(state)
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<Action> {
        let storage = state.storage(&self.tokens_storage);
        let place_list = TAPNPlaceListReader::from(storage);
        let mut actions = HashSet::new();
        for transi in self.transitions.iter() {
            let transi_actions = transi.available_actions(&place_list);
            actions.extend(transi_actions);
        }
        actions
    }

    fn available_delay(&self, state: &ModelState) -> RealTimeBound {
        let storage = state.storage(&self.tokens_storage);
        let place_list = TAPNPlaceListReader::from(storage);
        let mut min_delay = RealTimeBound::Infinite;
        for place in self.places.iter() {
            let tokens = place_list.place(place.index);
            let avail = place.available_delay(&tokens);
            if avail == RealTimeBound::MinusInfinite {
                return avail;
            }
            if avail < min_delay {
                min_delay = avail;
            }
        }
        min_delay
    }

    fn random_run<'a>(&'a self, initial : &'a ModelState, bound : VerificationBound)
        -> Box<dyn Iterator<Item = (Rc<ModelState>, ClockValue, Option<Action>)> + 'a>
    {
        Box::new(TAPNRunGenerator::generate(self, initial, bound))
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn is_timed(&self) -> bool {
        true
    }

    fn is_stochastic(&self) -> bool {
        false
    }

    fn init_initial_storage(&self, mut state : ModelState) -> ModelState {
        let n_places = self.places.len();
        let mut place_list = TAPNPlaceList {
            places : vec![ TAPNTokenList::new() ; n_places ]
        };
        for (i, place) in self.places.iter().enumerate() {
            let n_tokens = state.tokens(place.get_var());
            if n_tokens > 0 {
                let token = TAPNToken { count : n_tokens, age : ClockValue::zero() };
                place_list.places[i].push(token);
            }
        }
        *state.mut_storage(&self.tokens_storage) = ModelStorage::from(place_list);
        state
    }

    fn compile(&mut self, context : &mut ModelContext) -> CompilationResult<()> {
        self.places_dic.clear();
        self.id = context.new_model();
        self.tokens_storage = context.add_storage();
        let mut compiled_places = Vec::new();
        let mut places_down = vec![Vec::new() ; self.places.len()];
        let mut places_up = vec![Vec::new() ; self.places.len()];
        for place in self.places.iter() {
            let mut compiled_place = TAPNPlace::clone(&place);
            compiled_place.index = compiled_places.len();
            self.places_dic.insert(compiled_place.get_label(), compiled_place.index);
            compiled_place.compile(context)?;
            compiled_places.push(Arc::new(compiled_place));
        }
        self.places = compiled_places;
        let mut compiled_transitions = Vec::new();
        for transi in self.transitions.iter() {
            let mut compiled_transition = TAPNTransition::clone(&transi);
            compiled_transition.index = compiled_transitions.len();
            compiled_transition.compile(context)?;
            self.actions_dic.insert(compiled_transition.get_action(), compiled_transition.index);
            let transition = Arc::new(compiled_transition);
            self.create_transition_edges(&transition, &mut places_down, &mut places_up);
            compiled_transitions.push(transition);
        }
        self.transitions = compiled_transitions;
        for place in self.places.iter() {
            place.out_transitions.set(places_down[place.index].clone()).unwrap();
            place.in_transitions.set(places_up[place.index].clone()).unwrap();
        }
        Ok(())
    }

    fn nodes_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Node> + 'a> {
        let iter = self.places.iter().map(|p| p.as_node());
        let iter = iter.chain(self.transitions.iter().map(|t| t.as_node()));
        Box::new(iter)
    }

    fn edges(&self) -> Vec<Edge<String, Label, Label>> {
        let iter = self.transitions.iter().map(|t| {
            let iter = t.input_edges.get().unwrap().iter().map(Edge::stringify);
            iter.chain(t.output_edges.get().unwrap().iter().map(Edge::stringify))
        }).flatten();
        iter.collect()
    }

}

impl From<TAPNStructure> for TAPN {
    fn from(value: TAPNStructure) -> Self {
        TAPN::new(value.places, value.transitions)
    }
}

impl From<&TAPN> for TAPNStructure {
    fn from(value: &TAPN) -> Self {
        value.get_structure()
    }
}
