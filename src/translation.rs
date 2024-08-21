mod petri_class_graph;
mod petri_partial_observation;
use std::fmt::Display;

pub mod observation;

pub use petri_class_graph::PetriClassGraphTranslation;
pub use petri_partial_observation::PetriPartialObservation;

use crate::models::{lbl, model_context::ModelContext, Label, ModelObject, ModelState};

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
    OneByMany,
    CompleteOneByMany
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
pub struct TranslationChain {
    pub translations : Vec<Box<dyn Translation>>
}

impl Translation for TranslationChain {

    fn get_meta(&self) -> TranslationMeta {
        TranslationMeta {
            name : lbl("TranslationChain"),
            description : String::from("Structs used to chain translations into a more complex one."),
            input : match self.translations.first() {
                None => lbl("any"),
                Some(x) => x.get_meta().input
            },
            output : match self.translations.last() {
                None => lbl("any"),
                Some(x) => x.get_meta().output
            },
            translation_type : Unspecified,
        }
    }

    fn translate(&mut self, base : &dyn ModelObject, ctx : &ModelContext, initial_state : &ModelState) -> TranslationResult {
        if self.translations.is_empty() {
            return Err(TranslationError(String::from("Empty translation chain")));
        }
        let mut current_model = base;
        let mut initial_state = initial_state;
        let mut current_ctx = ctx;
        for translation in self.translations.iter_mut() {
            translation.translate(current_model, current_ctx, initial_state)?;
            (current_model, current_ctx, initial_state) = translation.get_translated();
        }
        Ok(())
    }

    fn get_translated(&mut self) -> (&mut dyn ModelObject, &ModelContext, &ModelState) {
        self.translations.last_mut().unwrap().get_translated()
    }

    fn back_translate(&self, state : ModelState) -> Option<ModelState> {
        let mut current_state = state;
        for translation in self.translations.iter().rev() {
            let back = translation.back_translate(current_state);
            match back {
                None => return None,
                Some(s) => current_state = s
            };
        }
        Some(current_state)
    }

    fn forward_translate(&self, state : ModelState) -> Option<ModelState> {
        let mut current_state = state;
        for translation in self.translations.iter() {
            let forward = translation.forward_translate(current_state);
            match forward {
                None => return None,
                Some(s) => current_state = s
            };
        }
        Some(current_state)
    }
    
    fn make_instance(&self) -> Box<dyn Translation> {
        Box::new(TranslationChain {
            translations : self.translations.iter().map(|t| t.make_instance()).collect()
        })
    }

}

