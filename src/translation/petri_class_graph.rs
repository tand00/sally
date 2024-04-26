use std::any::Any;

use crate::models::{class_graph::ClassGraph, lbl, petri::PetriNet, Model, ModelState};

use super::{Translation, TranslationMeta, TranslationType::SymbolicSpace};

use crate::log::*;

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

    fn get_meta(&self) -> TranslationMeta {
        TranslationMeta {
            name : lbl("PetriClassGraphTranslation"),
            description : String::from("Computes the class graph of a Time Petri Net"),
            input : lbl("TPN"),
            output : lbl("ClassGraph"),
            translation_type : SymbolicSpace,
        }
    }

    fn translate(&mut self, base : &dyn Any, initial_state : &ModelState) -> bool {
        pending("Computing Petri net Class graph...");
        let petri: Option<&PetriNet> = base.downcast_ref::<PetriNet>();
        if petri.is_none() {
            error("Unable to compute Class graph !");
            return false;
        }
        let petri = petri.unwrap();
        let graph = ClassGraph::from(petri, initial_state);
        positive("Class graph computed !");
        let mut initial_state = graph.classes[0].borrow().generate_image_state();
        let vars = initial_state.discrete.nrows();
        initial_state.discrete = initial_state.discrete.insert_row(vars, 0);
        self.initial_state = initial_state;
        self.class_graph = Some(graph);
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