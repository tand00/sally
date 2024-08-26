use std::collections::{HashMap, HashSet};

use num_traits::Zero;

use crate::verification::{smc::RandomRunIterator, VerificationBound};

use super::{action::{Action, ActionPairs}, lbl, model_context::ModelContext, time::{ClockValue, RealTimeBound}, CompilationResult, Label, Model, ModelMeta, ModelObject, ModelState, NONE};

pub struct ModelNetwork {
    pub id : usize,
    pub models : Vec<Box<dyn ModelObject>>,
    pub models_map : HashMap<Label, usize>,
    pub actions_map : HashMap<usize, usize>,
    pub io_actions : HashSet<Label, (Vec<Label>, Vec<Label>)>,
    pub sync_actions : HashMap<Action, ActionPairs>, // { Input : Output } s.t. (a => b) to fire
}

impl ModelNetwork {

    pub fn add_model(&mut self, name : Label, model : Box<dyn ModelObject>) {
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

    fn next(&self, state : ModelState, action : Action) -> Option<ModelState> {
        if !self.actions_map.contains_key(&action.get_id()) {
            return None;
        }
        let model_index = self.actions_map[&action.get_id()];
        let model = &self.models[model_index];
        let next = model.next(state, action);
        if next.is_none() {
            return None;
        }
        next
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<Action> {
        let mut actions = HashSet::new();
        for m in self.models.iter() {
            actions.extend(m.available_actions(state));
        }
        let mut synchros = HashSet::new();
        for (sync, pairs) in self.sync_actions.iter() {
            let enabled = pairs.enabled(&actions);
            actions = enabled.remove_io(actions);
            for (i,o) in enabled.generate_pairs() {
                synchros.insert(Action::Sync(sync.get_id(), Box::new(i), Box::new(o)));
            }
        }
        actions.extend(synchros);
        actions
    }

    fn available_delay(&self, state : &ModelState) -> RealTimeBound {
        let mut min_delay = RealTimeBound::Infinite;
        let mut is_timed = false;
        for model in self.models.iter() {
            if !model.is_timed() {
                continue
            }
            is_timed = true;
            let model_delay = model.available_delay(state);
            if model_delay < min_delay {
                min_delay = model_delay;
            }
        }
        if is_timed { 
            min_delay
        } else {
            RealTimeBound::zero()
        }
    }

    fn is_timed(&self) -> bool {
        self.models.iter().map(|m| m.is_timed() ).fold(true,|acc, x| acc || x)
    }

    fn is_stochastic(&self) -> bool {
        self.models.iter().map(|m| m.is_stochastic() ).fold(true,|acc, x| acc || x)
    }

    fn compile(&mut self, context : &mut ModelContext) -> CompilationResult<()> {
        for (name, model_index) in self.models_map.iter() {
            let model : &mut Box<dyn ModelObject> = &mut self.models[*model_index];
            context.add_domain(name.clone());
            model.compile(context)?;
            let model_actions = context.get_local_actions();
            for action in model_actions {
                self.actions_map.insert(action.get_id(), *model_index);
            }
            context.parent();
        }
        Ok(())
    }

    fn random_run<'a>(&'a self, initial : &'a ModelState, bound : VerificationBound) 
        -> Box<dyn Iterator<Item = (std::rc::Rc<ModelState>, ClockValue, Option<Action>)> + 'a> 
    {
        Box::new(RandomRunIterator::generate(self, initial, bound))
    }

    fn get_id(&self) -> usize {
        self.id
    }

}