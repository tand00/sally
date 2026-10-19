mod edge;
mod label;
mod model_state;
mod node;
mod model_context;
mod model_clock;
mod model_var;
mod model_storage;
mod model_project;
mod model_param;
mod action;

use std::{
    any::Any,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub use edge::Edge;
pub use label::{lbl, Label};
pub use model_state::ModelState;
pub use model_context::ModelContext;
pub use model_clock::ModelClock;
pub use model_var::*;
pub use model_storage::ModelStorage;
pub use model_project::ModelProject;
pub use model_param::ModelParam;
pub use action::*;
pub use node::Node;

use num_traits::Zero;
use time::RealTimeBound;

pub mod caching;
pub mod class_graph;
pub mod digraph;
pub mod expressions;
pub mod markov;
pub mod model_network;
pub mod model_solving_graph;
pub mod petri;
pub mod program;
pub mod run;
pub mod tapn;
pub mod time;
pub mod timed_automata;
pub mod beliefs_graph;

use crate::{computation::virtual_memory::EvaluationType, verification::VerificationBound};

use self::time::ClockValue;

#[derive(Debug, Clone)]
pub struct CompilationError;
pub type CompilationResult<T> = Result<T, CompilationError>;

pub type InitialMarking = HashMap<Label, EvaluationType>;

pub const UNMAPPED_ID : usize = usize::MAX;

#[derive(Debug, Clone, PartialEq)]
pub struct ModelMeta {
    pub name: Label,
    pub description: String,
}

impl std::fmt::Display for ModelMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " [.] Model ({})\n | - Description : \n | {}\n",
            self.name,
            self.description,
        )
    }
}

/// Generic trait that should be implemented by all Timed Transition Systems
pub trait Model: Any {
    // Given a state and an action, returns a state and actions available
    fn next(&self, state: ModelState, action: Action) -> Option<ModelState>;

    fn available_actions(&self, state: &ModelState) -> HashSet<Action>;

    fn available_delay(&self, state: &ModelState) -> RealTimeBound {
        let _ = state;
        if self.is_timed() {
            RealTimeBound::zero()
        } else {
            RealTimeBound::Infinite
        }
    }

    fn delay(&self, state: ModelState, dt: ClockValue) -> Option<ModelState> {
        let _ = dt;
        let _ = state;
        Some(state)
    }

    fn delay_next(&self, state: ModelState, dt: ClockValue, action: Action) -> Option<ModelState> {
        self.delay(state, dt)
            .and_then(|state| self.next(state, action))
    }

    fn init_initial_clocks(&self, state: ModelState) -> ModelState {
        state
    }

    fn init_initial_storage(&self, state: ModelState) -> ModelState {
        state
    }

    fn get_meta(&self) -> ModelMeta;

    fn is_timed(&self) -> bool;

    fn is_stochastic(&self) -> bool;

    // Default implementation of random_next sampler for SMC.
    // Should be overrided by stochastic models with a more relevant behaviour !
    fn random_next(&self, state: ModelState) -> (Option<ModelState>, ClockValue, Option<Action>) {
        let mut rng = rand::rng();
        let max_delay: ClockValue = self.available_delay(&state).into();
        let mut delayed_state = state;
        let mut delay = ClockValue::zero();
        if !max_delay.is_zero() && self.is_timed() {
            let delay_range = (ClockValue::zero())..(max_delay);
            delay = rng.gen_range(delay_range);
            delayed_state = self.delay(delayed_state, delay).unwrap();
        }
        let actions: Vec<Action> = self.available_actions(&delayed_state).into_iter().collect();
        let action = actions.choose(&mut rng);
        if action.is_none() {
            return (Some(delayed_state), delay, None);
        }
        let action = action.unwrap().clone();
        let next = self.next(delayed_state, action.clone());
        if next.is_none() {
            return (None, delay, Some(action));
        }
        (Some(next.unwrap()), delay, Some(action))
    }

    fn random_run<'a>(
        &'a self,
        initial: &'a ModelState,
        bound: VerificationBound,
    ) -> Box<dyn Iterator<Item = (Rc<ModelState>, ClockValue, Option<Action>)> + 'a>;

    fn compile(&mut self, context: &mut ModelContext) -> CompilationResult<()>;

    fn singleton(&mut self) -> CompilationResult<ModelContext> {
        let mut ctx = ModelContext::new();
        self.compile(&mut ctx)?;
        Ok(ctx)
    }

    fn get_id(&self) -> usize;

    fn is_compiled(&self) -> bool {
        self.get_id() != UNMAPPED_ID
    }

    fn nodes_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Node> + 'a>;

    fn edges(&self) -> Vec<Edge<String, Label, Label>>;

}

// Blanket implementation to add downgrading capabilities and calls to statics for each object implementing Model
pub trait ModelObject: Model + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self) -> Box<dyn Any>;
    fn model_object(&self) -> &dyn ModelObject;
}

impl<T: Model + Send + Sync> ModelObject for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn into_any(self) -> Box<dyn Any> {
        Box::new(self)
    }
    fn model_object(&self) -> &dyn ModelObject {
        self
    }
}

// Trait that should implement Send and Sync, to be shared amongst threads and do parallel verification by creating local models
pub trait ModelMaker<T: Model>: Send + Sync {
    fn create_maker(model: T) -> Self;

    fn make(&self) -> (T, ModelContext);
}
