pub mod models;
pub mod computation;
pub mod game;
pub mod translation;
pub mod verification;
pub mod solution;
pub mod log;

use std::collections::HashMap;

use models::lbl;
use models::petri::{PetriPlace, PetriTransition};
use models::time::{TimeInterval, TimeBound::*};
use solution::ClassGraphReachability;
use verification::query::{Condition, Expr, Query};

use crate::models::class_graph::ClassGraph;
use crate::models::model_solving_graph::ModelSolvingGraph;
use crate::models::petri::PetriNet;
use crate::translation::{PetriClassGraphTranslation, Translation};
use crate::models::Model;
use crate::solution::{ClassGraphReachabilitySynthesis, Solution};

use log::*;

//use models::class_graph::*;

fn main() {
    println!(" [#] Sally Model Checker - v.1.0");
    lf();
    println!(" [.] Features :");
    println!(" -> Models translation");
    println!(" -> Analytic solutions");
    println!(" -> Statistical Model Checking");
    println!(" -> Discrete verification");
    lf();

    pending("Building Model Solving Graph...");
    let solver = build_solver();
    positive(format!("Models loaded : \t[{}]", solver.models.len()));
    positive(format!("Translations : \t[{}]", solver.translations.len()));
    positive(format!("Solutions : \t[{}]", solver.solutions.len()));
    lf();

    let net = sample_petri();
    let query = sample_query();
    let mut translation = PetriClassGraphTranslation::new();
    let mut solution = ClassGraphReachability::new();
    let initial_state = net.get_initial_state(HashMap::from([
        (lbl("p0"), 1),
    ]));
    translation.translate(&net, &initial_state);
    let cg = translation.get_translated().0.downcast_ref::<ClassGraph>().unwrap();
    
    if solution.is_compatible(cg, &query) {
        positive("Solution compatible, ready to solve !");
        solution.solve(cg, &query);
    }

    lf();
    for c in cg.classes.iter() {
        println!("{}", c.borrow());
    }
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
    let condition = Condition::Evaluation(Expr::Object(4));
    Query::new(verification::query::Quantifier::Exists, verification::query::StateLogic::Finally, condition)
}