use std::{fmt::Display, sync::{OnceLock, Weak}};

use num_traits::Zero;

use crate::models::{action::Action, expressions::Condition, model_clock::ModelClock, model_context::ModelContext, time::ClockValue, CompilationError, CompilationResult, Edge, Label, ModelState};

use super::TAState;

#[derive(Debug, Default)]
pub struct TATransition {
    pub name : Label,
    pub guard : Condition,
    pub resets : Vec<ModelClock>,
    pub action : Action,
}

pub type TAEdge = Edge<TATransition, TAState, TAState>;

impl TATransition {

    pub fn new(name : Label, from : Label, to : Label) -> Self {
        TATransition { name, ..Default::default() }
    }

    pub fn set_resets(&mut self, resets : Vec<Label>) {
        self.resets = resets.into_iter().map(ModelClock::name).collect();
    }

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
        self.guard = cond.disjunctive_normal();
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

}

impl TAEdge {

    pub fn fire(&self, mut state : ModelState, cache : &usize) -> ModelState {
        for clock in self.data().resets.iter() {
            state.set_clock(clock, ClockValue::zero());
        }
        let source = self.get_node_from();
        let target = self.get_node_to();
        state.unmark(source.get_var(), 1);
        state.mark(target.get_var(), 1);
        let storage = state.mut_storage(cache);
        if storage.is_int() {
            let place_index = storage.mut_int();
            *place_index = target.index as i32;
        }
        state
    }

}

impl Clone for TATransition {
    fn clone(&self) -> Self {
        TATransition {
            name: self.name.clone(),
            guard: self.guard.clone(),
            resets: self.resets.clone(),
            ..Default::default()
        }
    }
}

impl Display for TATransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
