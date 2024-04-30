pub mod models;
pub mod computation;
pub mod game;
pub mod translation;
pub mod verification;
pub mod solution;
pub mod log;

use std::collections::HashMap;

use models::lbl;
use models::petri::{PetriPlace, PetriTransition, PetriStructure};
use models::time::{TimeInterval, TimeBound::*};
use solution::ClassGraphReachability;

use crate::models::class_graph::ClassGraph;
use crate::models::model_solving_graph::ModelSolvingGraph;
use crate::models::petri::PetriNet;
use crate::translation::{PetriClassGraphTranslation, Translation};
use crate::models::Model;
use crate::solution::{ClassGraphReachabilitySynthesis, Solution};
use crate::verification::query::*;

use log::*;

extern crate nalgebra as na;
extern crate num_traits;
extern crate rand;

//use models::class_graph::*;

fn main() {
    println!(" [#] Sally Model Checker - v.1.0");
    lf();
    println!(" [.] Features :");
    println!(" | - Models translation");
    println!(" | - Analytic solutions");
    println!(" | - Statistical Model Checking");
    println!(" | - Discrete verification");
    lf();

    pending("Building Model Solving Graph...");
    let solver = build_solver();
    positive(format!("Models loaded : \t[{}]", solver.models.len()));
    positive(format!("Translations : \t[{}]", solver.translations.len()));
    positive(format!("Solutions : \t[{}]", solver.solutions.len()));
    lf();

    let net = sample_petri();
    println!("{}", net.get_model_meta());
    lf();

    let mut query = sample_query();
    query.apply_to_model(&net);
    let mut translation = PetriClassGraphTranslation::new();
    let mut solution = ClassGraphReachability::new();
    let initial_state = net.get_initial_state(HashMap::from([
        (lbl("p0"), 1),
    ]));
    translation.translate(&net, &initial_state);
    let cg = translation.get_translated().0.downcast_ref::<ClassGraph>().unwrap();
    println!("{}", cg.get_model_meta());
    lf();
    
    if solution.is_compatible(cg, &query) {
        positive("Solution compatible, ready to solve !");
        solution.solve(cg, &query);
    }
    lf();

    for c in cg.classes.iter() {
        println!("{}", c.borrow());
    }
    let json_net = serde_json::to_string(&net.get_structure()).unwrap();
    println!("{}", json_net);

    let new_net : PetriStructure = serde_json::from_str(&json_net).unwrap();
    let new_net = PetriNet::from(new_net);
    println!("{}", new_net);

    let json_q = serde_json::to_string(&query).unwrap();
    println!("{}", json_q);
}

fn build_solver() -> ModelSolvingGraph {
    let mut solver = ModelSolvingGraph::new();
    solver.register_model(PetriNet::get_meta());
    solver.register_model(ClassGraph::get_meta());
    solver.register_translation(Box::new(PetriClassGraphTranslation::new()));
    solver.register_solution(Box::new(ClassGraphReachability::new()));
    solver.register_solution(Box::new(ClassGraphReachabilitySynthesis::new()));
    solver.compile();
    solver
}

fn sample_petri() -> PetriNet {
    let p0 = PetriPlace::new(lbl("p0"));
    let p1 = PetriPlace::new(lbl("p1"));
    let p2 = PetriPlace::new(lbl("p2"));
    let p3 = PetriPlace::new(lbl("p3"));
    let p4 = PetriPlace::new(lbl("p4"));
    let p5 = PetriPlace::new(lbl("p5"));
    let t0 = PetriTransition::new(
        lbl("t0"), 
        vec![lbl("p0")],
        vec![lbl("p1"), lbl("p4")], 
        TimeInterval(Large(0), Large(0))
    );
    let a = PetriTransition::new(
        lbl("a"), 
        vec![lbl("p1")],
        vec![lbl("p2")], 
        TimeInterval(Large(0), Large(4))
    );
    let b = PetriTransition::new(
        lbl("b"), 
        vec![lbl("p2"), lbl("p4")],
        vec![lbl("p3")],
        TimeInterval(Large(3), Large(4))
    );
    let c = PetriTransition::new(
        lbl("c"), 
        vec![lbl("p4")],
        vec![lbl("p5")],
        TimeInterval(Large(5), Large(6))
    );
    let net = PetriNet::new(
        vec![p0, p1, p2, p3, p4, p5], 
        vec![t0, a, b, c]
    );
    net
}

fn sample_query() -> Query {
    let condition = Condition::Or(
        Box::new(Condition::Evaluation(Expr::Name(lbl("p5")))),
        Box::new(Condition::Deadlock)
    );
    Query::new(Quantifier::Exists, StateLogic::Finally, condition)
}