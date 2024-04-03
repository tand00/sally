use super::Edge;
use super::petri::PetriClass;

pub struct ClassGraph {
    classes: Vec<PetriClass>,
    edges: Vec<Edge>
}