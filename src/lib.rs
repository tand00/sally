use io::sly::{SLYLoader, SLYWriter};
use models::{class_graph::ClassGraph, markov::markov_chain::MarkovChain, model_solving_graph::ModelSolvingGraph, petri::PetriNet, Model};
use solution::{ClassGraphReachability, ClassGraphReachabilitySynthesis};
use translation::PetriClassGraphTranslation;

pub mod models;
pub mod computation;
pub mod game;
pub mod learning;
pub mod translation;
pub mod verification;
pub mod solution;
pub mod io;
pub mod log;

pub fn sally_solver() -> ModelSolvingGraph {
    let mut solver = ModelSolvingGraph::new();
    solver.register_model(PetriNet::get_meta());
    solver.register_model(ClassGraph::get_meta());
    solver.register_model(MarkovChain::get_meta());
    solver.register_translation(PetriClassGraphTranslation::new());
    solver.register_solution(ClassGraphReachability::new());
    solver.register_solution(ClassGraphReachabilitySynthesis::new());
    solver.register_loader(SLYLoader);
    solver.register_writer(SLYWriter);
    solver
}