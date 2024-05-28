use std::{fmt, sync::{Arc, RwLock, Weak}};

use serde::{Serialize, Deserialize};

use crate::models::{model_context::ModelContext, model_var::{ModelVar, VarType}, CompilationResult, Label, Node};

use super::PetriTransition;

const PETRI_PLACE_VAR_TYPE : VarType = VarType::VarU8;

#[derive(Debug, Serialize, Deserialize)]
pub struct PetriPlace {
    pub name: Label,
    
    #[serde(skip)]
    pub index : usize,

    #[serde(skip)]
    in_transitions : RwLock<Vec<Weak<PetriTransition>>>,

    #[serde(skip)]
    out_transitions : RwLock<Vec<Weak<PetriTransition>>>,

    #[serde(skip)]
    data_variable : ModelVar
}

impl PetriPlace {

    pub fn new(lbl : Label) -> Self {
        PetriPlace {
            name: lbl,
            index : 0,
            in_transitions : RwLock::new(Vec::new()),
            out_transitions : RwLock::new(Vec::new()),
            data_variable: Default::default()
        }
    }

    pub fn add_upstream_transition(&self, transi : &Arc<PetriTransition>) {
        self.in_transitions.write().unwrap().push(Arc::downgrade(transi))
    }

    pub fn clear_upstream_transitions(&self) {
        self.in_transitions.write().unwrap().clear()
    }

    pub fn get_upstream_transitions(&self) -> Vec<Arc<PetriTransition>> {
        self.in_transitions.read().unwrap().iter().map(|pt| {
            Weak::upgrade(pt).unwrap()
        }).collect()
    }

    pub fn add_downstream_transition(&self, transi : &Arc<PetriTransition>) {
        self.out_transitions.write().unwrap().push(Arc::downgrade(transi))
    }

    pub fn clear_downstream_transitions(&self) {
        self.out_transitions.write().unwrap().clear()
    }

    pub fn get_downstream_transitions(&self) -> Vec<Arc<PetriTransition>> {
        self.out_transitions.read().unwrap().iter().map(|pt| {
            Weak::upgrade(pt).unwrap()
        }).collect()
    }

    pub fn set_var(&mut self, var : ModelVar) {
        self.data_variable = var;
    }

    pub fn get_var(&self) -> &ModelVar {
        &self.data_variable
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
            in_transitions : Default::default(),
            out_transitions: Default::default(),
            data_variable : Default::default()
        }
    }

}