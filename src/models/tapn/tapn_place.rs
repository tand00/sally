use std::{fmt, sync::{Arc, OnceLock}};

use num_traits::Zero;
use serde::{Serialize, Deserialize};

use crate::models::{model_context::ModelContext, model_var::{ModelVar, VarType}, time::{RealTimeBound, TimeBound}, CompilationResult, Label, ModelState, Node};
use super::{tapn_transition::TAPNTransition, TAPNTokenListReader};

const TAPN_PLACE_VAR_TYPE : VarType = VarType::VarU8;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TAPNPlace {
    pub name : Label,

    pub invariant : TimeBound,

    #[serde(skip)]
    pub index : usize,

    #[serde(skip)]
    pub in_transitions : OnceLock<Vec<Arc<TAPNTransition>>>,

    #[serde(skip)]
    pub out_transitions : OnceLock<Vec<Arc<TAPNTransition>>>,

    #[serde(skip)]
    data_variable : ModelVar
}

impl TAPNPlace {

    pub fn new(lbl : Label) -> Self {
        TAPNPlace {
            name : lbl,
            invariant : TimeBound::Infinite,
            index : 0,
            in_transitions : OnceLock::new(),
            out_transitions : OnceLock::new(),
            data_variable : Default::default()
        }
    }

    pub fn new_with_invariant(lbl : Label, inv : TimeBound) -> Self {
        TAPNPlace {
            name : lbl,
            invariant : inv,
            index : 0,
            in_transitions : OnceLock::new(),
            out_transitions : OnceLock::new(),
            data_variable : Default::default()
        }
    }

    pub fn clear_upstream_transitions(&mut self) {
        self.in_transitions = OnceLock::new()
    }

    pub fn get_upstream_transitions(&self) -> &Vec<Arc<TAPNTransition>> {
        self.in_transitions.get().unwrap()
    }

    pub fn clear_downstream_transitions(&mut self) {
        self.out_transitions = OnceLock::new()
    }

    pub fn get_downstream_transitions(&self) -> &Vec<Arc<TAPNTransition>> {
        self.out_transitions.get().unwrap()
    }

    pub fn set_var(&mut self, var : ModelVar) {
        self.data_variable = var;
    }

    pub fn get_var(&self) -> &ModelVar {
        &self.data_variable
    }

    pub fn n_tokens(&self, state : &ModelState) -> i32 {
        state.tokens(self.get_var())
    }

    pub fn available_delay(&self, tokens : &TAPNTokenListReader) -> RealTimeBound {
        let max_age = tokens.max_age();
        let inv = self.invariant.real();
        let translated = inv - max_age;
        if translated < RealTimeBound::zero() {
            RealTimeBound::MinusInfinite
        } else {
            translated
        }
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        self.set_var(ctx.add_var(self.get_label(), TAPN_PLACE_VAR_TYPE));
        Ok(())
    }

}

impl Node for TAPNPlace {

    fn get_label(&self) -> Label {
        self.name.clone()
    }

}

impl fmt::Display for TAPNPlace {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Place_{}", self.name)
    }

}

impl Clone for TAPNPlace {

    fn clone(&self) -> Self {
        TAPNPlace {
            name: self.name.clone(),
            index : self.index,
            invariant : self.invariant.clone(),
            ..Default::default()
        }
    }

}
