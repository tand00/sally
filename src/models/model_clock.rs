use serde::{Deserialize, Serialize};

use super::Label;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelClock {
    pub name : Label,
    pub index : usize
}

impl ModelClock {

    pub fn new() -> ModelClock {
        ModelClock {
            name : Label::new(), index : usize::MAX
        }
    }
    
    pub fn name(name : Label) -> ModelClock {
        ModelClock {
            name, index : usize::MAX
        }
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

}