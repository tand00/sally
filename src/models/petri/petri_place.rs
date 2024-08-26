use std::{fmt, sync::{Arc, OnceLock}};

use serde::{Serialize, Deserialize};

use crate::models::{model_context::ModelContext, model_var::{ModelVar, VarType}, CompilationResult, Label, ModelState, Node};

use super::PetriTransition;

const PETRI_PLACE_VAR_TYPE : VarType = VarType::VarU8;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PetriPlace {
    pub name: Label,

    #[serde(skip)]
    pub index : usize,

    #[serde(skip)]
    pub in_transitions : OnceLock<Vec<Arc<PetriTransition>>>,

    #[serde(skip)]
    pub out_transitions : OnceLock<Vec<Arc<PetriTransition>>>,

    #[serde(skip)]
    data_variable : ModelVar
}

impl PetriPlace {

    pub fn new(lbl : Label) -> Self {
        PetriPlace {
            name: lbl,
            index : 0,
            in_transitions : OnceLock::new(),
            out_transitions : OnceLock::new(),
            data_variable: Default::default()
        }
    }
    pub fn clear_upstream_transitions(&mut self) {
        self.in_transitions = OnceLock::new()
    }

    pub fn get_upstream_transitions(&self) -> &Vec<Arc<PetriTransition>> {
        self.in_transitions.get().unwrap()
    }

    pub fn clear_downstream_transitions(&mut self) {
        self.out_transitions = OnceLock::new()
    }

    pub fn get_downstream_transitions(&self) -> &Vec<Arc<PetriTransition>> {
        self.out_transitions.get().unwrap()
    }

    pub fn set_var(&mut self, var : ModelVar) {
        self.data_variable = var;
    }

    pub fn get_var(&self) -> &ModelVar {
        &self.data_variable
    }

    pub fn tokens(&self, state : &ModelState) -> i32 {
        state.tokens(self.get_var())
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        self.set_var(ctx.add_var(self.get_label(), PETRI_PLACE_VAR_TYPE));
        Ok(())
    }

}

impl Node for PetriPlace {

    fn get_label(&self) -> Label {
        self.name.clone()
    }

}

impl fmt::Display for PetriPlace {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Place_{}", self.name)
    }

}

impl Clone for PetriPlace {

    fn clone(&self) -> Self {
        PetriPlace {
            name: self.name.clone(),
            index : self.index,
            ..Default::default()
        }
    }

}
