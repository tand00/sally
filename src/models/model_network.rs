use std::collections::{HashMap, HashSet};

use super::{lbl, model_context::ModelContext, time::ClockValue, CompilationResult, Label, Model, ModelMeta, ModelState, NONE};

pub struct ModelNetwork {
    pub models : Vec<Box<dyn Model>>,
    pub models_map : HashMap<Label, usize>,
    pub model_vars_begin : Vec<usize>,
}

impl ModelNetwork {

    pub fn add_model(&mut self, name : Label, model : Box<dyn Model>) {
        self.models_map.insert(name, self.n_models());
        self.models.push(model);
    }

    pub fn n_models(&self) -> usize {
        self.models.len()
    }

}

impl Model for ModelNetwork {

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name : lbl("ModelNet"),
            description : String::from("Network of generic heterogeneous models"),
            characteristics : NONE
        }
    }

    fn next(&self, state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>) {
        (None, Default::default())
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<usize> {
        Default::default()
    }

    fn available_delay(&self, state : &ModelState) -> ClockValue {
        let mut min_delay = ClockValue::infinity();
        for model in self.models.iter() {
            let model_delay = model.available_delay(state);
            if model_delay < min_delay {
                min_delay = model_delay;
            }
        }
        min_delay
    }

    fn is_timed(&self) -> bool {
        self.models.iter().map(|m| m.is_timed() ).fold(true,|acc, x| acc || x)
    }

    fn is_stochastic(&self) -> bool {
        self.models.iter().map(|m| m.is_stochastic() ).fold(true,|acc, x| acc || x)
    }

    fn compile(&mut self, context : &mut ModelContext) -> CompilationResult<()> {
        for (name, model_index) in self.models_map.iter() {
            let model : &mut Box<dyn Model> = &mut self.models[*model_index];
            context.add_domain(name.clone());
            model.compile(context)?;
            context.parent();
        }
        Ok(())
    }

}