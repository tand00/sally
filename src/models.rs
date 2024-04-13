mod label;
mod node;
mod edge;
mod digraph;

pub use label::{lbl, Label};
pub use node::Node;
pub use edge::Edge;
pub use digraph::Digraph;

pub mod time;
pub mod petri;
pub mod observation;
pub mod class_graph;

use rand::seq::SliceRandom;

/// Generic trait that should be implemented by all Timed Transition Systems
pub trait Model {

    type State;
    type Action;
    
    // Given a state and an action, returns a (potentially incomplete) state, completion actions, and actions available
    fn next(&self, state : Self::State, action : Self::Action) -> (Option<Self::State>, Vec<Self::Action>);

    fn actions_available(&self, state : &Self::State) -> Vec<Self::Action>;

    fn random_run(&self, from : Self::State) {
        let mut actions = self.actions_available(&from);
        let mut state = Some(from);
        while state.is_some() {
            let s = state.unwrap();
            let action = actions.choose(&mut rand::thread_rng()).clone();
            if action.is_none() {
                break;
            }
            (state, actions) = self.next(s, *action.unwrap());
        }
    }

}

pub trait Timed : Model {

    fn available_delay(&self, state : &Self::State) -> f64;

    fn delay(&self, state : Self::State, dt : f64) -> Self::State;

}