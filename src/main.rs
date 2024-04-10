extern crate nalgebra as na;
extern crate num_traits;
extern crate serde_json as json;

pub mod models;
pub mod computation;
pub mod game;

use models::lbl;
use models::time::{TimeInterval, TimeBound::*};
use models::petri;

//use models::class_graph::*;

fn main() {
    let i0 = TimeInterval(Large(3), Infinite);
    let i1 = TimeInterval(Strict(3), Strict(5));
    let i2 = TimeInterval(MinusInfinite, Large(4));

    println!("{i0}");
    println!("{i1}");
    println!("{i2}");

    let t0 = petri::PetriTransition::new(
        lbl("t0"),
        vec![lbl("a")],
        vec![lbl("b")],
        i0
    );
    let t1 = petri::PetriTransition::new(
        lbl("t1"),
        vec![lbl("a")],
        vec![lbl("c")],
        i1
    );

    let place_a = petri::PetriPlace::new(lbl("a"));
    let place_b = petri::PetriPlace::new(lbl("b"));

    let model = petri::PetriNet::new(
        vec![place_a, place_b], 
        vec![t0, t1]);

    println!("{}", model);

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