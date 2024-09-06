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

use computation::probability::RealDistribution;
use io::sly::{SLYLoader, SLYWriter};
use io::tapn::TAPNLoader;
use models::model_project::ModelProject;
use models::lbl;
use models::markov::markov_chain::MarkovChain;
use models::petri::{PetriPlace, PetriTransition};
use models::tapn::tapn_edge::TAPNEdgeData;
use models::tapn::tapn_place::TAPNPlace;
use models::tapn::tapn_transition::TAPNTransition;
use models::tapn::TAPN;
use models::time::{Bound::{self, *}, TimeInterval};
use solution::ClassGraphReachability;

use crate::models::class_graph::ClassGraph;
use crate::models::model_solving_graph::ModelSolvingGraph;
use crate::models::petri::PetriNet;
use crate::translation::PetriClassGraphTranslation;
use crate::models::Model;
use crate::solution::ClassGraphReachabilitySynthesis;
use crate::verification::text_query_parser::parse_query;
use crate::verification::smc::{ProbabilityEstimation, SMCQueryVerification};

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

    let mut project = sample_tapn();//solver.load_file("test_petri.sly".to_owned()).unwrap_or(sample_petri());

    println!("{project}");

    project.compile().unwrap();

    let net = &project.model;
    println!("{}", net.get_model_meta());
    lf();

    let query = project.queries.first().unwrap();
    let initial_state = project.initial_state.clone().unwrap();

    println!("{:?}", initial_state);
    
    let mut prob_est = ProbabilityEstimation::fixed_runs(1000000, 0.95);
    prob_est.parallel_verify(net.model_object(), &initial_state, &query);

    /*let p_net = net.as_any().downcast_ref::<PetriNet>().unwrap();
    let cg = ClassGraph::compute(p_net, &initial_state);
    for class in cg.classes.iter() {
        println!("{}", class);
    }*/

    solver.write_file("test_petri.sly".to_owned(), &project).unwrap();
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
    solver.register_loader(TAPNLoader);
    solver.register_writer(SLYWriter);
    solver
}

fn sample_tapn() -> ModelProject {
    let p0 = TAPNPlace::new_with_invariant(lbl("p0"), Large(1));
    let mut t0 = TAPNTransition::new(
        lbl("t0"), 
        vec![
            (lbl("p0"), TAPNEdgeData {
                interval : TimeInterval::new(Large(0), Large(1)),
                weight: 1
            })
        ], Vec::new(), Vec::new(), Vec::new()
    );
    t0.distribution = RealDistribution::Uniform(0.0, 2.0);
    let tapn = TAPN::new(vec![p0], vec![t0]);
    let query = parse_query("G p0".to_owned()).unwrap();
    let marking = HashMap::from([(lbl("p0"), 1)]);
    ModelProject::new(Box::new(tapn), vec![query], marking)
}

fn sample_petri() -> ModelProject {
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
    let query = parse_query("F p5".to_owned()).unwrap();
    let marking = HashMap::from([(lbl("p0"), 1)]);
    ModelProject::new(Box::new(net), vec![query], marking)
}