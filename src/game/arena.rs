use std::collections::HashSet;

use crate::models::{action::Action, ModelObject};

pub struct Arena<'a> {

    pub model : &'a dyn ModelObject,
    pub players_actions : Vec<HashSet<Action>>

}
