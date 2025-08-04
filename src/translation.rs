 mod petri_class_graph;
mod petri_partial_observation;
use std::fmt::Display;

pub mod chain;
pub mod observation;

pub use petri_class_graph::PetriClassGraphTranslation;
pub use petri_partial_observation::PetriPartialObservation;

use crate::models::{ModelContext, Label, ModelObject, ModelState};

#[derive(Debug, Clone)]
pub struct TranslationError(pub String);
pub type TranslationResult = Result<(), TranslationError>;
impl Display for TranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Translation error : {}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationType {
    Unspecified,
    SymbolicSpace,
    Observation,
    Simulation,
    Bisimulation
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslationMeta {
    pub name : Label,
    pub description : String,
    pub input : Label,
    pub output : Label,
    pub translation_type : TranslationType
}

use TranslationType::*;

pub trait Translation {

    fn translate(&mut self, base : &dyn ModelObject, context : &ModelContext, initial_state : &ModelState) -> TranslationResult;

    fn get_translated(&mut self) -> (&mut dyn ModelObject, &ModelContext, &ModelState);

    fn get_meta(&self) -> TranslationMeta;

    fn is_stable(&self, state : &ModelState) -> bool {
        match self.back_translate(state.clone()) {
            Some(_) => true,
            None => false
        }
    }

    fn back_translate(&self, state : ModelState) -> Option<ModelState> {
        let _ = state;
        None
    }
    
    fn forward_translate(&self, state : ModelState) -> Option<ModelState> {
        let _ = state;
        None
    }

    fn make_instance(&self) -> Box<dyn Translation>;

}

pub trait TranslationFactory {

    fn get_meta(&self) -> TranslationMeta;

    fn make_instance(&self) -> Box<dyn Translation>;

    fn is_compatible(&self, model : &dyn ModelObject) -> bool;

}
