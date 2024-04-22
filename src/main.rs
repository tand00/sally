pub mod models;
pub mod computation;
pub mod game;
pub mod translation;
pub mod verification;
pub mod solution;

use crate::models::class_graph::ClassGraph;
use crate::models::model_solving_graph::ModelSolvingGraph;
use crate::models::petri::PetriNet;
use crate::translation::PetriClassGraphTranslation;
use crate::models::Model;
use crate::solution::ClassGraphReachabilitySynthesis;

//use models::class_graph::*;

fn main() {
    println!(" [#] Sally Model Checker - v.1.0");
    println!("");
    println!(" [.] Features :");
    println!(" -> Models translation");
    println!(" -> Analytic solutions");
    println!(" -> Statistical Model Checking");
    println!(" -> Discrete verification");
    println!("");

    println!(" [*] Building Model Solving Graph...");
    let solver = build_solver();
    println!(" [+] Models loaded : \t[{}]", solver.models.len());
    println!(" [+] Solutions : \t[{}]", solver.translations.len());
    println!(" [+] Translations : \t[{}]", solver.solutions.len());
    println!("");

    todo!("Bah faire la suite quoi mdr");
}

fn build_solver() -> ModelSolvingGraph {
    let mut solver = ModelSolvingGraph::new();
    solver.register_model(PetriNet::get_meta());
    solver.register_model(ClassGraph::get_meta());
    solver.register_translation(Box::new(PetriClassGraphTranslation::new()));
    solver.register_solution(Box::new(ClassGraphReachabilitySynthesis::new()));
    solver.compile();
    solver
}