use num_traits::Zero;

use crate::models::{action::Action, expressions::Condition, model_clock::ModelClock, model_context::ModelContext, time::ClockValue, CompilationError, CompilationResult, Label, ModelState};

#[derive(Debug, Clone)]
pub struct TATransition {
    pub name : Label,
    pub action : Action,
    pub guard : Condition,
    pub resets : Vec<ModelClock>
}

impl TATransition {

    pub fn get_name(&self) -> Label {
        self.name.clone()
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        self.action = ctx.add_action(self.get_name());
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

    pub fn fire(&self, mut state : ModelState) -> ModelState {
        for clock in self.resets.iter() {
            state.set_clock(clock, ClockValue::zero());
        }
        state
    }

}
