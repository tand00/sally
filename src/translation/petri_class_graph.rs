use crate::models::{class_graph::ClassGraph, lbl, model_context::ModelContext, petri::PetriNet, Model, ModelObject, ModelState};

use super::{Translation, TranslationError, TranslationMeta, TranslationResult, TranslationType::SymbolicSpace};

use crate::log;

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

    fn translate(&mut self, base : &dyn ModelObject, ctx : &ModelContext, initial_state : &ModelState) -> TranslationResult {
        log::pending("Computing Petri net Class graph...");
        self.context = ctx.clone();
        let petri: Option<&PetriNet> = base.as_any().downcast_ref::<PetriNet>();
        let Some(petri) = petri else {
            log::error("Unable to compute Class graph !");
            return Err(TranslationError(String::from("Cannot parse a Petri net from input parameter")));
        };
        let mut graph = ClassGraph::compute(petri, initial_state);
        if graph.compile(&mut self.context).is_err() {
            log::error("Unable to compile Class graph !");
            return Err(TranslationError(String::from("Cannot compile Petri net class graph")));
        }
        log::positive("Class graph computed !");
        let mut initial_state = graph.classes[0].generate_image_state();
        initial_state.discrete.size_delta(graph.current_class.size());
        self.initial_state = initial_state;
        self.class_graph = Some(graph);
        Ok(())
    }

    fn get_translated(&mut self) -> (&mut dyn ModelObject, &ModelContext, &ModelState) {
        (match &mut self.class_graph {
            None => panic!("No class graph computed !"),
            Some(cg) => cg
        }, &self.context, &self.initial_state)
    }

    fn make_instance(&self) -> Box<dyn Translation> {
        Box::new(Self::new())
    }



}
