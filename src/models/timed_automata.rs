use std::{collections::{HashMap, HashSet}, rc::Rc, sync::Arc};

mod ta_state;
mod ta_transition;

use num_traits::Zero;
pub use ta_state::TAState;
pub use ta_transition::TATransition;

use crate::verification::{smc::RandomRunIterator, VerificationBound};

use super::{action::Action, lbl, model_clock::ModelClock, model_context::ModelContext, model_storage::ModelStorage, time::{ClockValue, RealTimeBound}, CompilationResult, Model, ModelMeta, ModelState, CONTROLLABLE, TIMED};

pub struct TimedAutomata {
    pub id: usize,
    pub states: Vec<Arc<TAState>>,
    pub transitions: Vec<Arc<TATransition>>,
    pub clocks: Vec<ModelClock>,
    pub actions_dic: HashMap<Action, usize>,
    pub state_cache: usize // Store current active state, speed optimization at the cost of space. Might be worst
}

impl TimedAutomata {

    pub fn get_active_place(&self, state : &ModelState) -> &Arc<TAState> {
        let storage = state.storage(&self.state_cache);
        if storage.is_empty() {
            let index = state.argmax(self.states.iter().map(|s| s.get_var()));
            return &self.states[index];
        }
        let state_index = *storage.ref_int() as usize;
        &self.states[state_index]
    }

}

impl Model for TimedAutomata {

    fn next(&self, state: ModelState, action: Action) -> Option<ModelState> {
        let transi = self.actions_dic[&action];
        let transi = &self.transitions[transi];
        Some(transi.fire(state, &self.state_cache))
    }

    fn available_actions(&self, state: &ModelState) -> HashSet<Action> {
        let place = self.get_active_place(state);
        todo!()
    }

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name: lbl("TimedAutomata"),
            description: "Generic timed automata with multiple clocks".to_owned(),
            characteristics: TIMED | CONTROLLABLE
        }
    }

    fn is_timed(&self) -> bool {
        self.clocks.len() > 0
    }

    fn is_stochastic(&self) -> bool {
        false
    }

    fn random_run<'a>(&'a self, initial: &'a ModelState, bound: VerificationBound)
        -> Box<dyn Iterator<Item = (Rc<ModelState>, ClockValue, Option<Action>)> + 'a>
    {
        Box::new(RandomRunIterator::generate(self, initial, bound))
    }

    fn compile(&mut self, context: &mut ModelContext) -> CompilationResult<()> {
        self.id = context.new_model();
        for clock in self.clocks.iter_mut() {
            *clock = context.add_clock(clock.get_name())
        }
        let mut compiled_states = Vec::new();
        let mut places_dic = HashMap::new();
        for state in self.states.iter() {
            let mut compiled_state = TAState::clone(state);
            compiled_state.index = compiled_states.len();
            compiled_state.compile(context)?;
            let compiled_state = Arc::new(compiled_state);
            places_dic.insert(compiled_state.get_name(), Arc::clone(&compiled_state));
            compiled_states.push(compiled_state);
        }
        self.states = compiled_states;
        let mut compiled_transitions = Vec::new();
        self.actions_dic.clear();
        for transition in self.transitions.iter() {
            let mut compiled_transition = TATransition::clone(transition);
            compiled_transition.compile(context)?;
            self.actions_dic.insert(compiled_transition.get_action(), compiled_transitions.len());
            compiled_transitions.push(Arc::new(compiled_transition));
        }
        self.transitions = compiled_transitions;
        self.state_cache = context.add_storage();
        Ok(())
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn available_delay(&self, state: &ModelState) -> RealTimeBound {
        if !self.is_timed() {
            return RealTimeBound::Infinite;
        }
        let place = self.get_active_place(state);
        place.remaining_time(state)
    }

    fn delay(&self, mut state: ModelState, dt: ClockValue) -> Option<ModelState> {
        state.step_clocks(self.clocks.iter(), dt);
        let place = self.get_active_place(&state);
        if place.invariants.is_true(&state) {
            Some(state)
        } else {
            None
        }
    }

    fn init_initial_clocks(&self, mut state: ModelState) -> ModelState {
        for clock in self.clocks.iter() {
            state.enable_clock(clock, ClockValue::zero());
        }
        state
    }

    fn init_initial_storage(&self, mut state: ModelState) -> ModelState {
        let current_state = state.argmax(self.states.iter().map(|s| s.get_var()));
        let cache = state.mut_storage(&self.state_cache);
        *cache = ModelStorage::Integer(current_state as i32);
        state
    }
}
