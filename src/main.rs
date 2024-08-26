pub mod models;
pub mod computation;
pub mod game;
pub mod learning;
pub mod translation;
pub mod verification;
pub mod solution;
pub mod io;
pub mod log;

use std::collections::HashMap;

use computation::intervals::Convex;
use io::sly::{SLYLoader, SLYWriter};
use models::digraph::Digraph;
use models::expressions::{Condition, Expr};
use models::{lbl, ModelObject};
use models::markov::markov_chain::MarkovChain;
use models::markov::markov_node::MarkovNode;
use models::model_var::var;
use models::petri::{PetriPlace, PetriTransition, PetriStructure};
use models::time::{TimeInterval, Bound::*};
use solution::ClassGraphReachability;

use crate::models::class_graph::ClassGraph;
use crate::models::model_solving_graph::ModelSolvingGraph;
use crate::models::petri::PetriNet;
use crate::translation::{PetriClassGraphTranslation, Translation};
use crate::models::Model;
use crate::solution::{ClassGraphReachabilitySynthesis, Solution};
use crate::verification::text_query_parser::parse_query;
use crate::verification::{query::*, VerificationBound};
use crate::verification::smc::{ProbabilityEstimation, SMCMaxSeen, SMCQueryVerification};

use log::*;

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
    positive("Solver ready ! Loaded :");
    continue_info(format!("Semantics : \t[{}]", solver.semantics.len()));
    continue_info(format!("Solutions : \t[{}]", solver.solutions.len()));
    continue_info(format!("Loaders :   \t[{}]", solver.loaders.len()));
    continue_info(format!("Writers :   \t[{}]", solver.writers.len()));
    continue_info(format!("Translations : \t[{}]", solver.translations.len()));
    lf();

    let mut net = solver.load_file("test_petri.sly".to_owned())
        .map(|x| x.0)
        .unwrap_or(Box::new(sample_petri()));

    let ctx = net.singleton();
    println!("{}", ctx);
    println!("{}", net.get_model_meta());
    lf();
    
    let mut query = parse_query("F p5".to_owned()).unwrap();
    query.apply_to(&ctx).unwrap();
    let initial_state = ctx.make_initial_state(&*net, HashMap::from([(lbl("p0"), 1)]));
    
    let mut prob_est = ProbabilityEstimation::fixed_runs(1000000, 0.95);
    prob_est.parallel_verify(&*net, &initial_state, &query);

    let p_net = net.as_any().downcast_ref::<PetriNet>().unwrap();
    let cg = ClassGraph::compute(p_net, &initial_state);
    for class in cg.classes.iter() {
        println!("{}", class);
    }

    solver.write_file("test_petri.sly".to_owned(), &*net, None).unwrap();

    let q = parse_query("A F !(P3 && (P2 || P1 || P4)) || (P4 - P3 && !false || (P3 && (P2 && (P1 || V))))".to_owned()).unwrap();
    println!("{}", q.condition.disjunctive_normal());
    println!("{}", q.condition.conjunctive_normal());
    println!("{}", q.condition.distribute_not());
}

fn build_solver() -> ModelSolvingGraph {
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

fn sample_petri() -> PetriNet {
    let p0 = PetriPlace::new(lbl("p0"));
    let p1 = PetriPlace::new(lbl("p1"));
    let p2 = PetriPlace::new(lbl("p2"));
    let p3 = PetriPlace::new(lbl("p3"));
    let p4 = PetriPlace::new(lbl("p4"));
    let p5 = PetriPlace::new(lbl("p5"));
    let t0 = PetriTransition::safe(
        lbl("t0"),
        vec![lbl("p0")],
        vec![lbl("p1"), lbl("p4")],
        TimeInterval::new(Large(0), Large(0))
    );
    let a = PetriTransition::safe(
        lbl("a"),
        vec![lbl("p1")],
        vec![lbl("p2")],
        TimeInterval::new(Large(0), Large(4))
    );
    let b = PetriTransition::safe(
        lbl("b"),
        vec![lbl("p2"), lbl("p4")],
        vec![lbl("p3")],
        TimeInterval::new(Large(3), Large(4))
    );
    let c = PetriTransition::safe(
        lbl("c"),
        vec![lbl("p4")],
        vec![lbl("p5")],
        TimeInterval::new(Large(5), Large(6))
    );
    let net = PetriNet::new(
        vec![p0, p1, p2, p3, p4, p5],
        vec![t0, a, b, c]
    );
    net
}

fn _sample_digraph() -> Digraph<usize, i32> {
    let mut g : Digraph<usize, i32> = Digraph::new();
    g.make_node(3);
    g.make_node(4);
    g.make_node(2);
    g.make_node(1);
    g.make_edge(&3,&2, 1);
    g.make_edge(&3,&4, 3);
    g.make_edge(&2,&1, 5);
    g.make_edge(&4,&1, -1);
    g
}

fn sample_markov() -> MarkovChain {
    let m1 = MarkovNode::probabilistic(lbl("m1"), vec![
        (lbl("m2"), 0.8), (lbl("m3"), 0.2)
    ]);
    let m2 = MarkovNode::probabilistic(lbl("m2"), vec![
        (lbl("m2"), 1.0)
    ]);
    let m3 = MarkovNode::probabilistic(lbl("m3"), vec![
        (lbl("m3"), 1.0)
    ]);
    MarkovChain::new(vec![m1,m2,m3])
}
