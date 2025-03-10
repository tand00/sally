mod edge;
mod label;
mod model_state;
mod node;

use std::{
    any::Any,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub use edge::Edge;
pub use label::{lbl, Label};
pub use model_state::ModelState;
pub use node::Node;
use num_traits::Zero;
use rand::{seq::SliceRandom, thread_rng, Rng};
use time::RealTimeBound;

pub mod action;
pub mod caching;
pub mod class_graph;
pub mod digraph;
pub mod expressions;
pub mod markov;
pub mod model_clock;
pub mod model_const;
pub mod model_context;
pub mod model_network;
pub mod model_project;
pub mod model_solving_graph;
pub mod model_storage;
pub mod model_var;
pub mod petri;
pub mod program;
pub mod run;
pub mod tapn;
pub mod time;
pub mod timed_automata;
pub mod word;
pub mod beliefs_graph;
pub mod model_param;

use crate::{computation::virtual_memory::EvaluationType, verification::VerificationBound};

use self::{
    action::Action, model_characteristics::*, model_context::ModelContext, time::ClockValue,
};

#[derive(Debug, Clone)]
pub struct CompilationError;
pub type CompilationResult<T> = Result<T, CompilationError>;

pub type InitialMarking = HashMap<Label, EvaluationType>;

pub const UNMAPPED_ID : usize = usize::MAX;

pub mod model_characteristics {
    use crate::flag;

    use super::{lbl, Label};
    pub type ModelCharacteristics = u16;
    pub const NONE: ModelCharacteristics = 0;
    pub const TIMED: ModelCharacteristics = flag!(0);
    pub const CONTROLLABLE: ModelCharacteristics = flag!(1);
    pub const STOCHASTIC: ModelCharacteristics = flag!(2);
    pub const SYMBOLIC: ModelCharacteristics = flag!(3);

    pub fn has_characteristic(
        model_characteristics: ModelCharacteristics,
        characteristic: ModelCharacteristics,
    ) -> bool {
        (model_characteristics & characteristic) != 0
    }

    pub fn characteristics_label(model: ModelCharacteristics) -> Label {
        let mut characteritics: Vec<&str> = Vec::new();
        if model == 0 {
            return lbl("()");
        }
        if has_characteristic(model, TIMED) {
            characteritics.push("Timed");
        }
        if has_characteristic(model, CONTROLLABLE) {
            characteritics.push("Controllable");
        }
        if has_characteristic(model, STOCHASTIC) {
            characteritics.push("Stochastic");
        }
        if has_characteristic(model, SYMBOLIC) {
            characteritics.push("Symbolic");
        }
        Label::from(characteritics.join("|"))
    }
}

use model_characteristics::ModelCharacteristics;

#[derive(Debug, Clone, PartialEq)]
pub struct ModelMeta {
    pub name: Label,
    pub description: String,
    pub characteristics: ModelCharacteristics,
}
impl ModelMeta {
    pub fn is_timed(&self) -> bool {
        has_characteristic(self.characteristics, TIMED)
    }

    pub fn is_stochastic(&self) -> bool {
        has_characteristic(self.characteristics, STOCHASTIC)
    }
}

impl std::fmt::Display for ModelMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " [.] Model ({})\n | - Description : \n | {}\n | - Characteristics : {}",
            self.name,
            self.description,
            characteristics_label(self.characteristics)
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
        if let Some(delayed) = self.delay(state, dt) {
            self.next(delayed, action)
        } else {
            None
        }
    }

    fn init_initial_clocks(&self, state: ModelState) -> ModelState {
        state
    }

    fn init_initial_storage(&self, state: ModelState) -> ModelState {
        state
    }

    fn get_meta() -> ModelMeta
    where
        Self: Sized;

    fn is_timed(&self) -> bool;

    fn is_stochastic(&self) -> bool;

    // Default implementation of random_next sampler for SMC.
    // Should be overrided by stochastic models with a more relevant behaviour !
    fn random_next(&self, state: ModelState) -> (Option<ModelState>, ClockValue, Option<Action>) {
        let mut rng = thread_rng();
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

    fn edges(&self) -> Vec<Edge<String,Label,Label>>;

}

// Blanket implementation to add downgrading capabilities and calls to statics for each object implementing Model
pub trait ModelObject: Model + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn get_model_meta(&self) -> ModelMeta;
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
    fn get_model_meta(&self) -> ModelMeta {
        Self::get_meta()
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
