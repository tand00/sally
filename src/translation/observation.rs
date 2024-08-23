use std::{any::Any, cmp::max, collections::{HashMap, HashSet}};

use crate::{computation::virtual_memory::EvaluationType, models::{action::Action, lbl, model_clock::ModelClock, model_context::ModelContext, model_var::ModelVar, time::{ClockValue, RealTimeBound}, CompilationError, CompilationResult, Label, Model, ModelMeta, ModelObject, ModelState}, verification::Verifiable};
use crate::log;

use serde::{Deserialize, Serialize};
use VarObservationPolicy::*;

use super::{Translation, TranslationError, TranslationMeta, TranslationResult, TranslationType};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VarObservationPolicy {
    SumVars,
    MaxVar,
    UnitVar
}

impl Default for VarObservationPolicy {
    fn default() -> Self {
        SumVars
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ObservationFunction {
    pub vars : HashMap<Label, Label>,
    pub clocks : HashMap<Label, Label>,
    pub actions : HashMap<Label, Label>,
    #[serde(default)]
    pub var_policy : VarObservationPolicy,
}

pub struct PartialObservation<T : Model> {
    pub id : usize,
    pub observation_function : ObservationFunction,
    vars_link : Vec<(ModelVar, ModelVar)>,
    clocks_link : Vec<(ModelClock, ModelClock)>,
    actions_link : HashMap<Action, Action>,
    pub initial_context : ModelContext,
    pub context : ModelContext,
    pub model : Option<T>,
    pub initial_state : ModelState
}

impl<T : Model> PartialObservation<T> {

    pub fn new(obs : ObservationFunction) -> PartialObservation<T> {
        PartialObservation {
            id : usize::MAX,
            observation_function : obs,
            vars_link : Default::default(),
            clocks_link : Default::default(),
            actions_link : Default::default(),
            initial_context : Default::default(),
            context : Default::default(),
            model : Default::default(),
            initial_state : Default::default()
        }
    }

    pub fn create_context(&mut self, ctx : &ModelContext) {
        self.initial_context = ctx.clone();
        self.context = ModelContext::new();
        for _ in 0..ctx.n_models() {
            self.context.new_model();
        }
        for _ in 0..ctx.n_storages() {
            self.context.add_storage();
        }
        for var in ctx.get_vars() {
            let var_label = var.get_name();
            if !self.observation_function.vars.contains_key(&var_label) {
                continue;
            }
            let observed_label = self.observation_function.vars[&var_label].clone();
            let observed = self.context.get_or_add_var(observed_label, var.get_type());
            self.vars_link.push((var.clone(), observed));
        }
        for clock in ctx.get_clocks() {
            let clock_label = clock.get_name();
            if !self.observation_function.clocks.contains_key(&clock_label) {
                continue;
            }
            let observed_label = self.observation_function.clocks[&clock_label].clone();
            let observed = self.context.get_or_add_clock(observed_label);
            self.clocks_link.push((clock.clone(), observed));
        }
        for (action_name, action) in ctx.get_actions() {
            if !self.observation_function.actions.contains_key(&action_name) {
                continue;
            }
            let observed_label = self.observation_function.actions[&action_name].clone();
            let observed = self.context.get_or_add_action(observed_label);
            self.actions_link.insert(action.clone(), observed);
        }
    }

    pub fn observe(&self, state : &ModelState) -> ModelState {
        let mut observed = self.context.make_empty_state();
        let var_junction : fn(EvaluationType, EvaluationType) -> EvaluationType = 
        match self.observation_function.var_policy {
            SumVars => |x,y| x + y,
            MaxVar => |x,y| max(x, y),
            UnitVar => |x,_| if x > 0 { 1 } else { 0 }
        };
        for (x,o) in self.vars_link.iter() {
            let value = var_junction(state.evaluate_var(x), observed.evaluate_var(o));
            observed.set_marking(o, value);
        }
        for (x,o) in self.clocks_link.iter() {
            if state.is_enabled(x) {
                observed.set_clock(o, state.get_clock_value(x));
                break;
            }
        }
        observed.storages = state.storages.clone();
        observed.deadlocked = state.deadlocked;
        observed
    }

    pub fn observe_action(&self, action : &Action) -> Action {
        let base = action.base();
        if !self.actions_link.contains_key(&base) {
            return Action::Epsilon;
        }
        let result = self.actions_link[&base].clone();
        match action {
            Action::Epsilon => Action::Epsilon,
            Action::Internal(_) => result,
            Action::Sync(_, a, b) => result.sync(Action::clone(a), Action::clone(b)),
            Action::WithData(_, d) => result.with_data(d.clone())
        }
    }

}

impl<T : Model> Model for PartialObservation<T> {

    fn get_meta() -> ModelMeta {
        let sub_chars = T::get_meta();
        ModelMeta {
            name : lbl("PO-") + sub_chars.name,
            description : String::from("Partially observed model"),
            characteristics : sub_chars.characteristics
        }
    }

    fn next(&self, state : ModelState, action : Action) -> Option<ModelState> {
        todo!()
    }

    fn delay(&self, state : ModelState, dt : crate::models::time::ClockValue) -> Option<ModelState> {
        todo!()
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<Action> {
        todo!()
    }

    fn available_delay(&self, state : &ModelState) -> RealTimeBound {
        todo!()
    }

    fn is_stochastic(&self) -> bool {
        self.model.as_ref().unwrap().is_stochastic()
    }

    fn is_timed(&self) -> bool {
        self.model.as_ref().unwrap().is_timed()
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn compile(&mut self, context : &mut ModelContext) -> CompilationResult<()> {
        Err(CompilationError)
    }

}

impl<T : Model + Clone> Translation for PartialObservation<T> {
    
    fn get_meta(&self) -> TranslationMeta {
        TranslationMeta {
            name : lbl("PartialObservation"),
            description : String::from("Generic partial observation using model context"),
            input : lbl("any"),
            output : lbl("any"),
            translation_type : TranslationType::Observation,
        }
    }

    fn translate(&mut self, base : &dyn ModelObject, ctx : &ModelContext, initial_state : &ModelState) -> TranslationResult {
        log::pending("Computing Petri net Class graph...");
        let model = base.as_any().downcast_ref::<T>();
        if model.is_none() {
            return Err(TranslationError(String::from("Unable to downcast model")))
        }
        let model = model.unwrap();
        self.model = Some(model.clone());
        self.create_context(ctx);
        self.initial_state = self.observe(initial_state);
        Ok(())
    }

    fn get_translated(&mut self) -> (&mut dyn ModelObject, &ModelContext, &ModelState) {
        (match &mut self.model {
            None => panic!("No translation computed !"),
            Some(m) => m
        }, &self.context, &self.initial_state)
    }

    fn forward_translate(&self, state : ModelState) -> Option<ModelState> {
        Some(self.observe(&state))
    }
    
    fn make_instance(&self) -> Box<dyn Translation> {
        Box::new(Self::new(self.observation_function.clone()))
    }

}