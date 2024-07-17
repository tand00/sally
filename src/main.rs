pub mod models;
pub mod computation;
pub mod game;
pub mod learning;
pub mod translation;
pub mod verification;
pub mod solution;
pub mod log;

use std::collections::HashMap;

use computation::intervals::Convex;
use models::digraph::Digraph;
use models::expressions::{Condition, Expr};
use models::lbl;
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
    positive(format!("Models loaded : \t[{}]", solver.models.len()));
    positive(format!("Translations : \t[{}]", solver.translations.len()));
    positive(format!("Solutions : \t[{}]", solver.solutions.len()));
    lf();

    let mut net = sample_petri();
    let ctx = net.singleton();
    println!("{}", ctx);
    println!("{}", net.get_model_meta());
    lf();

    let mut translation = PetriClassGraphTranslation::new();
    let initial_state = ctx.make_initial_state(&net, HashMap::from([
        (lbl("p0"), 1),
    ]));
    translation.translate(&net, &ctx, &initial_state).unwrap();
    let (g, n_ctx, _) = translation.get_translated();
    let cg = g.downcast_ref::<ClassGraph>().unwrap();
    println!("{}", n_ctx);
    println!("{}", cg.get_model_meta());
    lf();

    for c in cg.classes.iter() {
        println!("{}", c);
        let json_m = serde_json::to_string(&c.dbm).unwrap();
        println!("{}", json_m);
    }

    let mut solution = ClassGraphReachability::new();
    let mut query = sample_query();
    query.apply_to(&ctx).unwrap();
    if solution.is_compatible(cg, &ctx, &query) {
        positive("Solution compatible, ready to solve !");
        solution.solve(cg, &ctx, &query);
    }
    lf();

    let mut estim  = ProbabilityEstimation::new(0.95, 0.05);
    let res = estim.verify(&net, &initial_state, &query);
    println!("{:?}", res);

    let mut estim  = ProbabilityEstimation::new(0.95, 0.05);
    let res = estim.parallel_verify(&net, &initial_state, &query);
    println!("{:?}", res);

    let mut estim  = ProbabilityEstimation::fixed_runs(100000, 0.95);
    let res = estim.parallel_verify(&net, &initial_state, &query);
    println!("{:?}", res);

    let estim  = SMCMaxSeen::new(100000);
    let res = estim.estimate_max(&net, &ctx, &initial_state, VerificationBound::StepsRunBound(1000));
    println!("{:?}", res);

    let res = estim.parallel_estimate_max(&net, &ctx, &initial_state, VerificationBound::StepsRunBound(1000));
    println!("{:?}", res);

    let json_net = serde_json::to_string(&net.get_structure()).unwrap();
    println!("{}", json_net);
    let new_net : PetriStructure = serde_json::from_str(&json_net).unwrap();
    let mut new_net = PetriNet::from(new_net);
    println!("{}", new_net.singleton());

    let json_q = serde_json::to_string(&query).unwrap();
    println!("{}", json_q);

    let q1 = parse_query(String::from("P <> [t <= 100] (P2 | deadlock) & P5 ^ 2 % 5")).unwrap();
    println!("-> {:#?}", q1);
    println!("-> {:#?}", serde_json::to_string(&q1).unwrap());

    let mut chain = sample_markov();
    let markov_ctx = chain.singleton();
    let state = markov_ctx.make_initial_state(&chain, HashMap::from([
        (lbl("m1"), 1),
    ]));
    let mut query = parse_query(String::from("P <> [# <= 10] m3")).unwrap();
    query.apply_to(&markov_ctx).unwrap();
    let mut estim  = ProbabilityEstimation::fixed_runs(100000, 0.95);
    let res = estim.parallel_verify(&chain, &state, &query);
    println!("{:?}", res);
    println!("{:?}", serde_json::to_string(&chain));

    let test = TimeInterval::new(Large(3),Strict(10));

    println!("{}", test);
    let test = test.intersection(TimeInterval::new(Large(1),Large(5)));
    println!("{}", test);
    let test = test.union(TimeInterval::new(Strict(7),Large(9)));
    println!("{}", test);
    let test = test.union(TimeInterval::new(Large(5),Strict(7)));
    println!("{}", test);
    let test = test.complement();
    println!("{}", test);
    let test = test.complement();
    println!("{}", test);
    let test = test.difference(TimeInterval::new(Large(4),Strict(8)));
    println!("{}", test);
}

fn build_solver() -> ModelSolvingGraph {
    let mut solver = ModelSolvingGraph::new();
    solver.register_model(PetriNet::get_meta());
    solver.register_model(ClassGraph::get_meta());
    solver.register_model(MarkovChain::get_meta());
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
    g.make_edge(3, 2, 1);
    g.make_edge(3, 4, 3);
    g.make_edge(2, 1, 5);
    g.make_edge(4, 1, -1);
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

fn sample_query() -> Query {
    let condition = Condition::And(
        Box::new(Condition::Evaluation(Expr::Var(var("p5")))),
        Box::new(Condition::Deadlock)
    );
    Query::new(Quantifier::Exists, StateLogic::Finally, condition)
}
