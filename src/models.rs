mod label;
mod node;
mod edge;
mod model_state;

use std::{any::Any, cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

pub use label::{lbl, Label};
pub use model_state::ModelState;
pub use node::Node;
pub use edge::Edge;
//pub use digraph::Digraph;
use num_traits::Zero;
use rand::{thread_rng, Rng, seq::SliceRandom};

pub mod time;
pub mod expressions;
pub mod program;
pub mod petri;
pub mod class_graph;
pub mod model_solving_graph;
pub mod digraph;
pub mod model_network;
//pub mod markov_chain;
pub mod run;

use self::{model_characteristics::*, time::ClockValue};

#[derive(Debug, Clone)]
pub struct CompilationError;
pub type CompilationResult<T> = Result<T, CompilationError>;

pub type ComponentPtr<T> = Rc<RefCell<T>>;
pub fn new_ptr<T>(x : T) -> ComponentPtr<T> {
    Rc::new(RefCell::new(x))
}

pub mod model_characteristics {
    use crate::flag;

    use super::{lbl, Label};
    pub type ModelCharacteristics = u16;
    pub const NONE : ModelCharacteristics = 0;
    pub const TIMED : ModelCharacteristics = flag!(0);
    pub const CONTROLLABLE : ModelCharacteristics = flag!(1);
    pub const STOCHASTIC : ModelCharacteristics = flag!(2);
    pub const SYMBOLIC : ModelCharacteristics = flag!(3);

    pub fn has_characteristic(model_characteristics : ModelCharacteristics, characteristic : ModelCharacteristics) -> bool {
        (model_characteristics & characteristic) != 0
    }

    pub fn characteristics_label(model : ModelCharacteristics) -> Label {
        let mut characteritics : Vec<&str> = Vec::new();
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
    name : Label,
    description : String,
    characteristics : ModelCharacteristics,
}
impl ModelMeta {
    pub fn is_timed(&self) -> bool where Self : Sized {
        has_characteristic(self.characteristics, TIMED)
    }

    pub fn is_stochastic(&self) -> bool where Self : Sized {
        has_characteristic(self.characteristics, STOCHASTIC)
    }
}
impl std::fmt::Display for ModelMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " [.] Model ({})\n | Description : \n | {}\n | Characteristics : {}", self.name, self.description, characteristics_label(self.characteristics))
    }
}

/// Generic trait that should be implemented by all Timed Transition Systems
pub trait Model : Any {
    
    // Given a state and an action, returns a state and actions available
    fn next(&self, state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>);

    fn available_actions(&self, state : &ModelState) -> HashSet<usize>;

    fn available_delay(&self, state : &ModelState) -> ClockValue {
        let _ = state;
        ClockValue::zero()
    }

    fn n_vars(&self) -> usize;

    fn delay(&self, state : ModelState, dt : ClockValue) -> Option<ModelState> {
        let _ = dt;
        let _ = state;
        None
    }

    fn init_initial_clocks(&self, state : ModelState) -> ModelState {
        state
    }

    fn get_meta() -> ModelMeta where Self : Sized;

    fn get_model_meta(&self) -> ModelMeta where Self : Sized { // Same as before but instance
        Self::get_meta()
    }

    fn is_timed(&self) -> bool;

    fn is_stochastic(&self) -> bool;

    // Default implementation of random_next sampler for SMC. 
    // Should be overrided by stochastic models with a more relevant behaviour !
    fn random_next(&self, state : ModelState) -> (Option<ModelState>, ClockValue, Option<usize>) {
        let mut rng = thread_rng();
        let max_delay = self.available_delay(&state);
        let mut delayed_state = state;
        let mut delay = ClockValue::zero();
        if !max_delay.is_zero() {
            let delay_range = (ClockValue::zero())..(max_delay);
            delay = rng.gen_range(delay_range);
            delayed_state = self.delay(delayed_state, delay).unwrap();
        }
        let actions : Vec<usize> = self.available_actions(&delayed_state).into_iter().collect();
        let action = actions.choose(&mut rng);
        if action.is_none() {
            return (Some(delayed_state), delay, None)
        }
        let action = *action.unwrap();
        let (next, _) = self.next(delayed_state, action);
        (next, delay, Some(action))
    }

    fn get_initial_state(&self, marking : HashMap<Label, i32>) -> ModelState {
        let mut state = ModelState::new(self.n_vars(), 0);
        for (k,v) in marking.iter() {
            let index = self.map_label_to_var(k);
            if index.is_none() {
                continue;
            }
            let index = index.unwrap();
            state.discrete[index] = *v;
        }
        self.init_initial_clocks(state)
    }

    fn map_label_to_var(&self, var : &Label) -> Option<usize>;

    fn compile(&mut self) -> CompilationResult<()> {
        Ok(())
    }

}