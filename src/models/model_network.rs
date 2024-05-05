use std::collections::{HashMap, HashSet};

use super::{lbl, time::ClockValue, Label, Model, ModelMeta, ModelState, NONE};

pub struct ModelNetwork {
    pub models : Vec<Box<dyn Model>>,
    pub models_map : HashMap<Label, usize>,
    pub model_vars_begin : Vec<usize>,
}

impl ModelNetwork {

    pub fn add_model(&mut self, name : Label, model : Box<dyn Model>) {
        self.models_map.insert(name, self.n_models());
        self.model_vars_begin.push(model.n_vars());
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
        self.models.iter().map(|m| m.is_timed() ).fold(true,|acc, x| acc && x)
    }

    fn is_stochastic(&self) -> bool {
        self.models.iter().map(|m| m.is_stochastic() ).fold(true,|acc, x| acc && x)
    }

    fn map_label_to_var(&self, var : &Label) -> Option<usize> {
        let (net_name, model_var) = var.get_first_ident();
        if model_var.is_empty() || !self.models_map.contains_key(&net_name) {
            return None;
        }
        let model_index = self.models_map[&net_name];
        let model = &self.models[model_index];
        match model.map_label_to_var(&model_var) {
            None => None,
            Some(i) => Some(self.model_vars_begin[model_index] + i)
        }
    }

    fn n_vars(&self) -> usize {
        if self.model_vars_begin.len() == 0 {
            return 0;
        }
        *self.model_vars_begin.last().unwrap()
    }

}