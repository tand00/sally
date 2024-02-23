pub mod dbm;
pub mod models;
pub mod game;

use std::collections::HashMap;

use models::{Model, State, Transition};
use models::lbl;
use models::time::{TimeInterval, TimeBound::*};
use models::petri;

//use models::class_graph::*;

use crate::game::Run;

fn main() {
    let i0 = TimeInterval(Large(3), Infinite);
    let i1 = TimeInterval(Strict(3), Strict(5));
    let i2 = TimeInterval(MinusInfinite, Large(4));

    println!("{i0}");
    println!("{i1}");
    println!("{i2}");

    let t0 = petri::PetriTransition {
        label: lbl("t0"),
        from: vec![lbl("a")],
        to: vec![lbl("b")],
        interval: i0
    };
    let t1 = petri::PetriTransition {
        label: lbl("t1"),
        from: vec![lbl("a")],
        to: vec![lbl("c")],
        interval: i1
    };

    let state_a = petri::PetriState(lbl("a"));
    let state_b = petri::PetriState(lbl("b"));

    let mut r = Run::empty();
    r.push_state(state_a.clone_box());
    r.push_action(t0.clone_box());
    r.push_state(state_b.clone_box());
    println!("Run : {r}");

    let mut model = petri::PetriNet {
        states: vec![state_a, state_b],
        transitions: vec![t0, t1],
        initial_states: vec![lbl("a")],
    };

    println!("{}", model);
    
    let mut result = model.check_labels_coherence();

    println!("{result}");

    model.states.push(petri::PetriState(lbl("c")));

    result = model.check_labels_coherence();

    println!("{result}");

    /*let mut s = ClassGraph {
        classes: vec![
            ClassGraphClass { sub_states: vec![lbl("a"),lbl("b")], domain: HashMap::new() },
            ClassGraphClass { sub_states: vec![lbl("a"),lbl("b")], domain: HashMap::new() },
        ],
        transitions: vec![
            ClassGraphTransition { interval: TimeInterval(Large(0), Infinite), from:  }
        ],
        initial_states: vec![lbl("a")],
    };
    let c = s.compile();
    //s.states = Vec::new();
    s.classes = Vec::new();*/

}