use std::{sync::{Arc, OnceLock}, usize};

use crate::models::{expressions::Condition, model_context::ModelContext, model_var::{ModelVar, VarType}, time::RealTimeBound, CompilationError, CompilationResult, Label, ModelState, Node};

use super::ta_transition::TAEdge;

const TA_STATE_VAR_TYPE : VarType = VarType::VarU8;

#[derive(Debug, Default)]
pub struct TAState {
    pub name : Label,
    pub invariants : Condition,
    pub var : ModelVar,
    pub accepting : bool,
    pub downsteam : OnceLock<Vec<Arc<TAEdge>>>,
    pub upstream : OnceLock<Vec<Arc<TAEdge>>>,
    pub index : usize
}

impl TAState {

    pub fn new(name : Label) -> Self {
        TAState { name, accepting: false, ..Default::default() }
    }

    pub fn with_invariants(name : Label, invariants : Condition) -> Self {
        TAState { name, invariants, accepting: false, ..Default::default() }
    }

    pub fn get_name(&self) -> Label {
        self.name.clone()
    }

    pub fn get_var(&self) -> &ModelVar {
        &self.var
    }

    pub fn remaining_time(&self, state : &ModelState) -> RealTimeBound {
        let conds = self.invariants.conjunctions();
        let max_time = RealTimeBound::Infinite;
        for cond in conds {
            todo!()
        }
        max_time
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        self.var = ctx.add_var(self.get_name(), TA_STATE_VAR_TYPE);
        if !self.invariants.is_clock_guard() {
            return Err(CompilationError);
        }
        let Ok(cond) = self.invariants.apply_to(ctx) else {
            return Err(CompilationError);
        };
        self.invariants = cond.disjunctive_normal();
        Ok(())
    }

}

impl Node for TAState {
    fn get_label(&self) -> Label {
        self.get_name()
    }
}

impl Clone for TAState {
    fn clone(&self) -> Self {
        TAState {
            name : self.name.clone(),
            invariants : self.invariants.clone(),
            index : self.index,
            ..Default::default()
        }
    }
}
