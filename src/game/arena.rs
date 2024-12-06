use std::collections::HashSet;

use crate::models::{action::Action, ModelObject};

pub struct Arena {
    pub model : Box<dyn ModelObject>,
    pub players_actions : Vec<HashSet<Action>>

}
