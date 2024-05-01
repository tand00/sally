mod petri_class_graph;
mod petri_partial_observation;
use std::any::Any;

pub use petri_class_graph::PetriClassGraphTranslation;
pub use petri_partial_observation::PetriPartialObservation;

use crate::models::{lbl, Label, Model, ModelState};

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

    fn translate(&mut self, base : &dyn Any, initial_state : &ModelState) -> bool;

    fn get_translated(&mut self) -> (&mut dyn Any, &ModelState);
    fn get_translated_model(&mut self) -> (&mut dyn Model, &ModelState);

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

    fn translate(&mut self, base : &dyn Any, initial_state : &ModelState) -> bool {
        if self.translations.is_empty() {
            return false;
        }
        let mut current_model = base;
        let mut initial_state = initial_state;
        for translation in self.translations.iter_mut() {
            let result = translation.translate(current_model, initial_state);
            if !result {
                return false;
            }
            (current_model, initial_state) = translation.get_translated();
        }
        true
    }

    fn get_translated(&mut self) -> (&mut dyn Any, &ModelState) {
        self.translations.last_mut().unwrap().get_translated()
    }

    fn get_translated_model(&mut self) -> (&mut dyn Model, &ModelState) {
        self.translations.last_mut().unwrap().get_translated_model()
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

}
