use std::collections::HashMap;

use crate::models::model_var::ModelVar;
use crate::models::action::Action;

pub struct IOContext {
    pub input_actions : HashMap<u32, Action>,
    pub output_actions : HashMap<Action, u32>,
    pub variables_connections : HashMap<ModelVar, u32>
}

pub struct ElectronicsMachine {
    
}