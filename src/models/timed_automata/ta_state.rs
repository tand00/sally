use std::sync::{Arc, OnceLock};

use crate::models::{expressions::Condition, model_context::ModelContext, model_var::{ModelVar, VarType}, time::RealTimeBound, CompilationError, CompilationResult, Label, ModelState};

use super::TATransition;

const TA_STATE_VAR_TYPE : VarType = VarType::VarU8;

#[derive(Debug,Clone)]
pub struct TAState {
    pub name : Label,
    pub invariants : Condition,
    pub var : ModelVar,
    pub downsteam : OnceLock<Vec<Arc<TATransition>>>,
    pub upstream : OnceLock<Vec<Arc<TATransition>>>,
    pub index : usize
}

impl TAState {

    pub fn get_name(&self) -> Label {
        self.name.clone()
    }

    pub fn get_var(&self) -> &ModelVar {
        &self.var
    }

    pub fn remaining_time(&self, state : &ModelState) -> RealTimeBound {
        todo!()
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        self.var = ctx.add_var(self.get_name(), TA_STATE_VAR_TYPE);
        if !self.invariants.is_clock_guard() {
            return Err(CompilationError);
        }
        let Ok(cond) = self.invariants.apply_to(ctx) else {
            return Err(CompilationError);
        };
        self.invariants = cond;
        Ok(())
    }

}
