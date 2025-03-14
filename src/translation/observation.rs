use std::{cmp::max, collections::{HashMap, HashSet}};

use function::{ObservationContext, ObservationFunction, VarPolicy};
use observable::Observable;

use crate::{computation::virtual_memory::EvaluationType, models::{action::Action, class_graph::StateClass, lbl, model_clock::ModelClock, model_context::ModelContext, model_var::ModelVar, time::{ClockValue, RealTimeBound}, CompilationError, CompilationResult, Label, Model, ModelMeta, ModelObject, ModelState, UNMAPPED_ID}, verification::{smc::RandomRunIterator, Verifiable, VerificationBound}};
use crate::log;

use super::{Translation, TranslationError, TranslationMeta, TranslationResult, TranslationType};

pub mod function;
pub mod observable;

pub struct PartialObservation<T : Model> {
    pub id : usize,
    pub observation_function : ObservationFunction,
    pub obs_ctx : ObservationContext,
    pub model : Option<T>,
    pub initial_state : ModelState
}

impl<T : Model> PartialObservation<T> {

    pub fn new(obs : ObservationFunction) -> PartialObservation<T> {
        PartialObservation {
            id : UNMAPPED_ID,
            observation_function : obs,
            obs_ctx : Default::default(),
            model : Default::default(),
            initial_state : Default::default()
        }
    }

    pub fn observe(&self, state : &ModelState) -> ModelState {
        state.observe(&self.obs_ctx, &self.observation_function)
    }

    pub fn observe_action(&self, action : &Action) -> Action {
        action.observe(&self.obs_ctx, &self.observation_function)
    }

}

/*impl<T : Model> Model for PartialObservation<T> {

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

    fn delay(&self, state : ModelState, dt : ClockValue) -> Option<ModelState> {
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

    fn random_run<'a>(&'a self, initial : &'a ModelState, bound : VerificationBound)
        -> Box<dyn Iterator<Item = (std::rc::Rc<ModelState>, ClockValue, Option<Action>)> + 'a>
    {
        Box::new(RandomRunIterator::generate(self, initial, bound))
    }

}*/

impl<T : ModelObject + Clone> Translation for PartialObservation<T> {

    fn get_meta(&self) -> TranslationMeta {
        TranslationMeta {
            name : lbl("PartialObservation"),
            description : String::from("Generic partial observation using model context"),
            input : T::get_meta().name,
            output : lbl("PO-") + T::get_meta().name,
            translation_type : TranslationType::Observation,
        }
    }

    fn translate(&mut self, base : &dyn ModelObject, ctx : &ModelContext, initial_state : &ModelState) -> TranslationResult {
        log::pending("Computing observation context...");
        let model = base.as_any().downcast_ref::<T>();
        if model.is_none() {
            return Err(TranslationError(String::from("Unable to downcast model")))
        }
        let model = model.unwrap();
        self.model = Some(model.clone());
        self.obs_ctx = self.observation_function.generate_context(ctx.clone());
        self.initial_state = self.observe(initial_state);
        Ok(())
    }

    fn get_translated(&mut self) -> (&mut dyn ModelObject, &ModelContext, &ModelState) {
        (match &mut self.model {
            None => panic!("No translation computed !"),
            Some(m) => m
        }, &self.obs_ctx.observed, &self.initial_state)
    }

    fn forward_translate(&self, state : ModelState) -> Option<ModelState> {
        Some(self.observe(&state))
    }

    fn make_instance(&self) -> Box<dyn Translation> {
        Box::new(Self::new(self.observation_function.clone()))
    }

}
