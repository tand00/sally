mod label;
mod node;
mod edge;

pub use label::{lbl, Label};
pub use node::Node;
pub use edge::Edge;

pub mod time;
pub mod petri;
pub mod observation;
pub mod class_graph;

/// Generic trait that should be implemented by all types of models
pub trait Model {

    type State;
    type Action;
    
    fn next(&self, state : &Self::State, action : Self::Action) -> Self::State;

}