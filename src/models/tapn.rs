use std::{collections::HashSet, iter::zip, sync::Arc};

use num_traits::Zero;
use tapn_place::TAPNPlace;
use tapn_token::*;
use tapn_transition::TAPNTransition;

use super::{action::Action, lbl, model_context::ModelContext, model_storage::ModelStorage, time::ClockValue, CompilationResult, Model, ModelMeta, ModelState, CONTROLLABLE, TIMED};

pub mod tapn_place;
pub mod tapn_edge;
pub mod tapn_transition;
pub mod tapn_token;

pub struct TAPN {
    pub id : usize,
    pub storage_index : usize,
    pub places : Vec<Arc<TAPNPlace>>,
    pub transitions : Vec<Arc<TAPNTransition>>,
}

impl TAPN {

    pub fn fire(&self, mut state : ModelState, transi : usize, in_tokens : TAPNPlaceList) -> (ModelState, HashSet<usize>) {
        let mut places_tokens = TAPNPlaceListAccessor::from(state.mut_storage(&self.storage_index));
        let transi = &(self.transitions[transi]);
        let mut modified_places = HashSet::new();
        let mut vars_updates = Vec::new();
        for edge in transi.input_edges.read().unwrap().iter() {
            let place = edge.get_node_from();
            vars_updates.push((place.clone(), -edge.data().weight));
            let state_tokens = &mut places_tokens.places[place.index]; 
            let input_tokens = &in_tokens.places[place.index];
            state_tokens.remove_set(input_tokens);
        }
        for edge in transi.output_edges.read().unwrap().iter() {
            let target = edge.get_node_to();
            vars_updates.push((target.clone(), edge.data().weight));
            let target_tokens = &mut places_tokens.places[target.index];
            target_tokens.insert(TAPNToken { count: edge.data().weight, age: ClockValue::zero() });
        }
        for edge in transi.transports.read().unwrap().iter() {
            let place = edge.get_node_from();
            let target = edge.get_node_to();
            vars_updates.push((place.clone(), -edge.data().weight));
            vars_updates.push((target.clone(), edge.data().weight));
            let state_tokens = &mut places_tokens.places[place.index]; 
            let input_tokens = &in_tokens.places[place.index];
            state_tokens.remove_set(input_tokens);
            let target_tokens = &mut places_tokens.places[target.index];
            for token in input_tokens.iter() {
                target_tokens.insert(token.clone());
            }
        }
        for (place, delta) in vars_updates {
            state.mark(place.get_var(), delta);
            modified_places.insert(place.index);
        }
        (state, modified_places)
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

    fn next(&self, mut state : ModelState, action : Action) -> Option<(ModelState, HashSet<Action>)> {
        let storage = state.mut_storage(&self.storage_index);
        let mut placeList = TAPNPlaceListAccessor::from(storage);

        None
    }

    fn delay(&self, mut state : ModelState, dt : ClockValue) -> Option<ModelState> {
        let storage = state.mut_storage(&self.storage_index);
        let mut place_list = TAPNPlaceListAccessor::from(storage);
        place_list.delta(dt);
        Some(state)
    }

    fn random_next(&self, state : ModelState) -> (Option<ModelState>, ClockValue, Option<Action>) {
        (None, ClockValue::zero(), None)
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<Action> {
        HashSet::new()
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
        *state.mut_storage(&self.storage_index) = ModelStorage::from(place_list);
        state
    }

    fn compile(&mut self, context : &mut ModelContext) -> CompilationResult<()> {
        self.id = context.new_model();
        self.storage_index = context.add_storage();
        let mut compiled_places = Vec::new();
        for place in self.places.iter() {
            let mut compiled_place = TAPNPlace::clone(&place);
            compiled_place.compile(context)?;
            compiled_places.push(Arc::new(compiled_place));
        }
        self.places = compiled_places;
        let mut compiled_transitions = Vec::new();
        for transi in self.transitions.iter() {
            let mut compiled_transition = TAPNTransition::clone(&transi);
            compiled_transition.compile(context)?;
            compiled_transitions.push(Arc::new(compiled_transition));
        }
        self.transitions = compiled_transitions;
        Ok(())
    }

}