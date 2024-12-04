use std::{collections::HashSet, rc::Rc, sync::Arc};

mod ta_state;
mod ta_transition;

use num_traits::Zero;
pub use ta_state::TAState;
pub use ta_transition::TATransition;

use crate::verification::{smc::RandomRunIterator, VerificationBound};

use super::{action::Action, lbl, model_clock::ModelClock, model_context::ModelContext, model_var::{ModelVar, VarType}, time::{ClockValue, RealTimeBound}, CompilationResult, Model, ModelMeta, ModelState, CONTROLLABLE, TIMED};

pub struct TimedAutomata {
    pub id: usize,
    pub states: Vec<Arc<TAState>>,
    pub transitions: Vec<Arc<TATransition>>,
    pub clocks: Vec<ModelClock>,
}

impl TimedAutomata {}

impl Model for TimedAutomata {

    fn next(&self, state: ModelState, action: Action) -> Option<ModelState> {
        todo!()
    }

    fn available_actions(&self, state: &ModelState) -> HashSet<Action> {
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
        let mut compiled_transitions = Vec::new();
        for transition in self.transitions.iter() {
            let mut compiled_transition = TATransition::clone(transition);
            compiled_transition.compile(context)?;
            compiled_transitions.push(Arc::new(compiled_transition));
        }
        self.transitions = compiled_transitions;
        let mut compiled_states = Vec::new();
        for state in self.states.iter() {
            let mut compiled_state = TAState::clone(state);
            compiled_state.compile(context)?;
            compiled_states.push(Arc::new(compiled_state));
        }
        self.states = compiled_states;
        Ok(())
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn available_delay(&self, state: &ModelState) -> RealTimeBound {
        if !self.is_timed() {
            return RealTimeBound::Infinite;
        }
        todo!()
    }

    fn delay(&self, state: ModelState, dt: ClockValue) -> Option<ModelState> {
        let _ = dt;
        let _ = state;
        Some(state)
    }

    fn init_initial_clocks(&self, mut state: ModelState) -> ModelState {
        for clock in self.clocks.iter() {
            state.enable_clock(clock, ClockValue::zero());
        }
        state
    }
}
