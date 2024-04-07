use super::Edge;
use super::petri::{PetriClass, PetriMarking, PetriNet};

pub struct ClassGraph {
    classes: Vec<PetriClass>,
    edges: Vec<Edge>
}

impl ClassGraph {

    pub fn from(p_net : PetriNet, initial_marking : PetriMarking) -> Self {
        let mut cg = ClassGraph {
            classes: Vec::new(),
            edges: Vec::new()
        };
        cg
    }

}