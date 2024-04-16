mod label;
mod node;
mod edge;
mod digraph;
mod model_state;

use std::collections::HashSet;

pub use label::{lbl, Label};
pub use model_state::ModelState;
pub use node::Node;
pub use edge::Edge;
pub use digraph::Digraph;
use num_traits::Zero;

pub mod time;
pub mod petri;
pub mod observation;
pub mod class_graph;
pub mod model_solving_graph;
pub mod translation;

use crate::verification::decidable_solutions::DecidableSolution;

use self::{model_characteristics::*, node::SimpleNode, time::ClockValue};

pub mod model_characteristics {
    use crate::flag;
    pub type ModelCharacteristics = u16;
    pub const NONE : ModelCharacteristics = 0;
    pub const TIMED : ModelCharacteristics = flag!(0);
    pub const CONTROLLABLE : ModelCharacteristics = flag!(1);
    pub const STOCHASTIC : ModelCharacteristics = flag!(2);

    pub fn has_characteristic(model_characteristics : ModelCharacteristics, characteristic : ModelCharacteristics) -> bool {
        (model_characteristics & characteristic) != 0
    }
}

use model_characteristics::ModelCharacteristics;

#[derive(Clone, PartialEq)]
pub struct ModelMeta {
    name : Label,
    solutions : DecidableSolution,
    characteristics : ModelCharacteristics,
    //translations : Vec<Label> Translations will be added one by one
}
impl std::fmt::Display for ModelMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Model_{}_{}_{}", self.name, self.characteristics, self.solutions)
    }
}

/// Generic trait that should be implemented by all Timed Transition Systems
pub trait Model {
    
    // Given a state and an action, returns a state and actions available
    fn next(&self, state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>);

    fn actions_available(&self, state : &ModelState) -> HashSet<usize>;

    fn available_delay(&self, state : &ModelState) -> ClockValue {
        ClockValue::zero()
    }

    fn n_vars(&self) -> usize;
    
    fn n_clocks(&self) -> usize {
        0
    }

    fn delay(&self, state : ModelState, dt : ClockValue) -> Option<ModelState> {
        None
    }

    fn get_meta() -> ModelMeta;

    fn get_model_meta(&self) -> ModelMeta { // Same as before but instance
        Self::get_meta()
    }

    fn is_timed(&self) -> bool {
        has_characteristic(Self::get_meta().characteristics, TIMED)
    }

}