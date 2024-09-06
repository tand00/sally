use std::{collections::HashMap, fmt::Display};

use crate::verification::Query;

use super::{model_context::ModelContext, CompilationError, CompilationResult, InitialMarking, ModelObject, ModelState};

pub struct ModelProject {
    pub model : Box<dyn ModelObject>,
    pub queries : Vec<Query>,
    pub initial_marking : InitialMarking,
    pub initial_state : Option<ModelState>
}

impl ModelProject {

    pub fn new(model : Box<dyn ModelObject>, queries : Vec<Query>, initial_marking : InitialMarking) -> ModelProject {
        ModelProject {
            model,
            queries,
            initial_marking,
            initial_state : None
        }
    }

    pub fn only_model(model : Box<dyn ModelObject>) -> ModelProject {
        ModelProject {
            model,
            queries : Vec::new(),
            initial_marking : HashMap::new(),
            initial_state : None
        }
    }

    pub fn compile(&mut self) -> CompilationResult<ModelContext> {
        let mut ctx = self.model.singleton()?;
        for query in self.queries.iter_mut() {
            if query.apply_to(&mut ctx).is_err() {
                return Err(CompilationError);
            }
        }
        self.initial_state = Some(ctx.make_initial_state(&*self.model, self.initial_marking.clone()));
        Ok(ctx)
    }

}

impl Display for ModelProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " [.] Model Project :\n")?;
        write!(f, " | - Model type : {}\n", self.model.get_model_meta().name)?;
        write!(f, " | - Queries : [{}]", self.queries.len())
    }
}