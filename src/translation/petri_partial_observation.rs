use std::any::Any;

use crate::models::{class_graph::ClassGraph, lbl, petri::PetriNet, Model, ModelState};

use super::{Translation, TranslationMeta, TranslationResult, TranslationType::Observation};

use crate::log::*;

pub struct PetriPartialObservation {
    pub initial_state : ModelState,
}

impl PetriPartialObservation {
    pub fn new() -> Self {
        PetriPartialObservation {
            initial_state : ModelState::new(0, 0),
        }
    }
}

impl Translation for PetriPartialObservation {

    fn get_meta(&self) -> TranslationMeta {
        TranslationMeta {
            name : lbl("PetriPartialObservation"),
            description : String::from("Partially observe a Time Petri Net"),
            input : lbl("TPN"),
            output : lbl("POTPN"),
            translation_type : Observation,
        }
    }

    fn translate(&mut self, base : &dyn Any, initial_state : &ModelState) -> TranslationResult {
        panic!("Nothing for now")
    }

    fn get_translated(&mut self) -> (&mut dyn Any, &ModelState) {
        panic!("Nothing for now")
    }

    fn get_translated_model(&mut self) -> (&mut dyn Model, &ModelState) {
        panic!("Nothing for now")
    }

}