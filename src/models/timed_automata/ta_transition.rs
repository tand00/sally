use std::sync::{OnceLock, Weak};

use num_traits::Zero;

use crate::models::{action::Action, expressions::Condition, model_clock::ModelClock, model_context::ModelContext, time::ClockValue, CompilationError, CompilationResult, Label, ModelState};

use super::TAState;

#[derive(Debug, Clone)]
pub struct TATransition {
    pub name : Label,
    pub from : Label,
    pub to : Label,
    pub action : Action,
    pub guard : Condition,
    pub resets : Vec<ModelClock>,
    pub node_from : OnceLock<Weak<TAState>>,
    pub node_to : OnceLock<Weak<TAState>>,
}

impl TATransition {

    pub fn get_name(&self) -> Label {
        self.name.clone()
    }

    pub fn get_action(&self) -> Action {
        self.action.clone()
    }

    pub fn merge_target_invariants(&mut self, target : &TAState) {
        let mut cond = target.invariants.clone();
        for clock in self.resets.iter() {
            cond = cond.remove_clock(clock);
        }
        self.guard &= cond
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        self.action = ctx.add_action(self.get_name());
        if !self.guard.is_clock_guard() {
            return Err(CompilationError);
        }
        let Ok(cond) = self.guard.apply_to(ctx) else {
            return Err(CompilationError)
        };
        self.guard = cond;
        for clock in self.resets.iter_mut() {
            let Ok(c) = clock.apply_to(ctx) else {
                return Err(CompilationError);
            };
            *clock = c;
        }
        Ok(())
    }

    pub fn is_enabled(&self, state : &ModelState) -> bool {
        self.guard.is_true(state)
    }

    pub fn fire(&self, mut state : ModelState, cache : &usize) -> ModelState {
        for clock in self.resets.iter() {
            state.set_clock(clock, ClockValue::zero());
        }
        let source = self.node_from.get().unwrap().upgrade().unwrap();
        let target = self.node_to.get().unwrap().upgrade().unwrap();
        state.unmark(source.get_var(), 1);
        state.mark(target.get_var(), 1);
        let place_index = state.mut_storage(cache).mut_int();
        *place_index = target.index as i32;
        state
    }

}
