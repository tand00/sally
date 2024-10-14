use std::collections::HashMap;

use code_translator::CodeTranslator;

use crate::computation::probability::RealDistribution;
use crate::models::model_project::ModelProject;
use crate::models::model_var::ModelVar;
use crate::models::action::Action;

pub mod code_translator;

pub struct IOContext {
    pub input_actions : HashMap<u32, Action>,
    pub output_actions : HashMap<Action, u32>,
    pub input_vars : HashMap<u32, ModelVar>,
    pub output_vars : HashMap<ModelVar, u32>
}

pub struct ElectronicsMachine {
    pub project : ModelProject,
    pub inputs_distributions : HashMap<u32, RealDistribution>,
    pub io_context : IOContext,
    pub update_rate : f64
}

impl ElectronicsMachine {

    pub fn export_code(&self, exporter : &impl CodeTranslator) -> String {
        todo!()        
    }

}