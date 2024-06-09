use std::{collections::HashSet, sync::Arc};

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

    pub fn fire(&self, mut state : ModelState, transi : usize, in_tokens : TAPNPlaceList) -> (Option<ModelState>, HashSet<usize>, HashSet<usize>) {
        
        todo!()
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
        Ok(())
    }

}