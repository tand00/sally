use std::any::Any;

use crate::models::{class_graph::ClassGraph, lbl, model_context::ModelContext, petri::PetriNet, Model, ModelState};

use super::{Translation, TranslationError, TranslationMeta, TranslationResult, TranslationType::SymbolicSpace};

use crate::log::*;

pub struct PetriClassGraphTranslation {
    pub initial_state : ModelState,
    pub context : ModelContext,
    pub class_graph : Option<ClassGraph>,
}

impl PetriClassGraphTranslation {
    pub fn new() -> Self {
        PetriClassGraphTranslation {
            initial_state : ModelState::new(0, 0),
            context : ModelContext::new(),
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

    fn translate(&mut self, base : &dyn Any, ctx : &ModelContext, initial_state : &ModelState) -> TranslationResult {
        pending("Computing Petri net Class graph...");
        self.context = ctx.clone();
        let petri: Option<&PetriNet> = base.downcast_ref::<PetriNet>();
        if petri.is_none() {
            error("Unable to compute Class graph !");
            return Err(TranslationError(String::from("Cannot parse a Petri net from input parameter")));
        }
        let petri = petri.unwrap();
        let mut graph = ClassGraph::compute(petri, initial_state);
        let compilation_res = graph.compile(&mut self.context);
        if compilation_res.is_err() {
            error("Unable to compile Class graph !");
            return Err(TranslationError(String::from("Cannot compile Petri net class graph")));
        }
        positive("Class graph computed !");
        let mut initial_state = graph.classes[0].borrow().generate_image_state();
        initial_state.discrete.size_delta(graph.current_class.size());
        self.initial_state = initial_state;
        self.class_graph = Some(graph);
        Ok(())
    }

    fn get_translated(&mut self) -> (&mut dyn Any, &ModelContext, &ModelState) {
        (match &mut self.class_graph {
            None => panic!("No class graph computed !"),
            Some(cg) => cg
        }, &self.context, &self.initial_state)
    }

    fn get_translated_model(&mut self) -> (&mut dyn Model, &ModelContext, &ModelState) {
        (match &mut self.class_graph {
            None => panic!("No class graph computed !"),
            Some(cg) => cg
        }, &self.context, &self.initial_state)
    }

}