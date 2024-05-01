mod label;
mod node;
mod edge;
mod model_state;

use std::{any::Any, cell::RefCell, collections::HashSet, rc::Rc};

pub use label::{lbl, Label};
pub use model_state::ModelState;
pub use node::Node;
pub use edge::Edge;
//pub use digraph::Digraph;
use num_traits::Zero;

pub mod time;
pub mod petri;
pub mod class_graph;
pub mod model_solving_graph;
pub mod digraph;
pub mod markov_chain;

use self::{model_characteristics::*, time::ClockValue};

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
impl std::fmt::Display for ModelMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " [.] Model ({})\n | Description : \n | {}\n | Characteristics : {}", self.name, self.description, characteristics_label(self.characteristics))
    }
}

/// Generic trait that should be implemented by all Timed Transition Systems
pub trait Model : Any {
    
    // Given a state and an action, returns a state and actions available
    fn next(&self, state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>);

    fn actions_available(&self, state : &ModelState) -> HashSet<usize>;

    fn available_delay(&self, state : &ModelState) -> ClockValue {
        let _ = state;
        ClockValue::zero()
    }

    fn n_vars(&self) -> usize;
    
    fn n_clocks(&self) -> usize {
        0
    }

    fn delay(&self, state : ModelState, dt : ClockValue) -> Option<ModelState> {
        let _ = dt;
        let _ = state;
        None
    }

    fn get_meta() -> ModelMeta where Self : Sized;

    fn get_model_meta(&self) -> ModelMeta where Self : Sized { // Same as before but instance
        Self::get_meta()
    }

    fn is_timed(&self) -> bool where Self : Sized {
        has_characteristic(Self::get_meta().characteristics, TIMED)
    }

    fn map_label_to_var(&self, var : &Label) -> Option<usize>;

}