use std::collections::HashSet;

use crate::models::{action::Action, ModelObject};

use super::strategy::Strategy;

pub struct Player {
    pub strategy : Box<dyn Strategy>,
    pub actions : HashSet<Action>
}

pub struct Arena<'a> {
    pub model : &'a dyn ModelObject,
    pub players : Vec<Player>
}
