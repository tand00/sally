use std::any::Any;

use crate::models::{class_graph::ClassGraph, lbl, petri::PetriNet, Model, ModelState};

use super::{Translation, TranslationMeta, TranslationType::SymbolicSpace};

pub struct PetriClassGraphTranslation {
    pub initial_state : ModelState,
    pub class_graph : Option<ClassGraph>,
}

impl PetriClassGraphTranslation {
    pub fn new() -> Self {
        PetriClassGraphTranslation {
            initial_state : ModelState::new(0, 0),
            class_graph : None,
        }
    }
}

impl Translation for PetriClassGraphTranslation {

    fn get_meta() -> TranslationMeta {
        TranslationMeta {
            name : lbl("PetriClassGraphTranslation"),
            description : String::from("Computes the class graph of a Time Petri Net"),
            input : lbl("TimePetriNet"),
            output : lbl("ClassGraph"),
            translation_type : SymbolicSpace,
        }
    }

    fn translate(&mut self, base : &dyn Any, initial_state : &ModelState) -> bool {
        let petri: Option<&PetriNet> = base.downcast_ref::<PetriNet>();
        if petri.is_none() {
            return false;
        }
        let petri = petri.unwrap();
        self.class_graph = Some(ClassGraph::from(petri, initial_state));
        true
    }

    fn get_translated(&mut self) -> (&mut dyn Any, &ModelState) {
        (match &mut self.class_graph {
            None => panic!("No class graph computed !"),
            Some(cg) => cg
        }, &self.initial_state)
    }

    fn get_translated_model(&mut self) -> (&mut dyn Model, &ModelState) {
        (match &mut self.class_graph {
            None => panic!("No class graph computed !"),
            Some(cg) => cg
        }, &self.initial_state)
    }

}