use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::{model_context::ModelContext, model_var::{MappingError, MappingResult}, Label};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelClock {
    pub name : Label,
    pub index : Option<usize>
}

impl ModelClock {

    pub fn new() -> ModelClock {
        ModelClock {
            name : Label::new(), index : None
        }
    }

    pub fn name(name : Label) -> ModelClock {
        ModelClock {
            name, index : None
        }
    }

    pub fn get_name(&self) -> Label {
        self.name.clone()
    }

    pub fn get_index(&self) -> Option<usize> {
        self.index
    }

    pub fn is_mapped(&self) -> bool {
        self.index.is_some()
    }

    pub fn apply_to(&self, ctx : &ModelContext) -> MappingResult<ModelClock> {
        let res = ctx.get_clock(&self.name);
        match res {
            None => Err(MappingError(Label::from("Unable to map clock to index !"))),
            Some(v) => Ok(v)
        }
    }

}

impl Display for ModelClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl Default for ModelClock {

    fn default() -> Self {
        ModelClock::new()
    }

}
