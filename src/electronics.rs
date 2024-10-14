use std::collections::HashMap;

use crate::models::model_var::ModelVar;
use crate::models::action::Action;

pub mod code_translator;

pub struct IOContext {
    pub input_actions : HashMap<u32, Action>,
    pub output_actions : HashMap<Action, u32>,
    pub variables_connections : HashMap<ModelVar, u32>
}

pub struct ElectronicsMachine {
    pub project : ModelProject,
    pub io_context : IOContext,
    pub inputs_distributions : HashMap<u32, RealDistribution>,
    pub io_events : HashSet<u32>,
    pub io_measures : HashSet<u32>
}

impl ElectronicsMachine {

    pub fn export_code(&self) -> String {

    }

}