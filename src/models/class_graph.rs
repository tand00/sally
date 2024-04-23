mod state_class;
pub use state_class::StateClass;

use core::panic;
use std::collections::{HashMap, HashSet, VecDeque};

use num_traits::Zero;

use crate::computation::DBM;

use super::time::ClockValue;
use super::{lbl, Edge, Label, Model, ModelMeta, ModelState, CONTROLLABLE, SYMBOLIC, TIMED};
use super::petri::PetriNet;

const CLASS_LIMIT : usize = 4096;

pub struct ClassGraph {
    pub classes: Vec<StateClass>,
    pub edges: Vec<Edge<usize>>,
    pub n_clocks : usize,
    pub places_dic : HashMap<Label, usize>,
}

impl ClassGraph {

    pub fn from(p_net : &PetriNet, initial_state : &ModelState) -> Self {
        let mut cg = ClassGraph {
            classes: Vec::new(),
            edges: Vec::new(),
            n_clocks : p_net.n_clocks(),
            places_dic : p_net.places_dic.clone()
        };
        let mut seen : HashMap<u64, usize> = HashMap::new();
        let mut to_see : VecDeque<usize> = VecDeque::new();
        let initial_class = StateClass::compute_class(p_net, initial_state);
        seen.insert(initial_class.get_hash(), 0);
        cg.classes.push(initial_class);
        to_see.push_back(0);
        while !to_see.is_empty() {
            let class_index = to_see.pop_back().unwrap();
            let class = &cg.classes[class_index];
            let clocks = class.enabled_clocks();
            let accessible : Vec<(StateClass, usize)> = clocks.iter().filter_map(|action| {
                match ClassGraph::successor(p_net, &class, *action) {
                    None => None,
                    Some(c) => Some((c, *action))
                }
            }).collect();
            for (new_class, action) in accessible { // Ugly but to avoid immut ref of cg existing in loop
                let mut new_class : StateClass = new_class;
                let new_hash = new_class.get_hash();
                if seen.contains_key(&new_hash) {
                    cg.classes[seen[&new_hash]].predecessors.push((class_index, action));
                    continue;
                }
                let new_index = cg.classes.len();
                new_class.predecessors.push((class_index, action));
                seen.insert(new_hash, new_index);
                cg.classes.push(new_class);
                to_see.push_back(new_index);
                if cg.classes.len() > CLASS_LIMIT {
                    panic!("Class limit overflow ! Petri net may not be bounded !");
                }
            }
        }
        cg.make_edges();
        cg
    }

    pub fn successor(petri : &PetriNet, class : &StateClass, action : usize) -> Option<StateClass> {
        let image_state = class.generate_image_state();
        let (next_state, newen, pers) = petri.fire(image_state, action);

        let vars = newen.len() + pers.len();
        let mut next_dbm = DBM::new(vars);
        let mut to_dbm : Vec<usize> = vec![0 ; petri.transitions.len()];
        let mut from_dbm : Vec<usize> = vec![0];
        let fired_i = class.to_dbm_index[action];
        let discrete = next_state.discrete;

        for transi in 0..petri.transitions.len() {
            if pers.contains(&transi) {
                let dbm_index = from_dbm.len();
                to_dbm[transi] = dbm_index;
                from_dbm.push(transi);
                let previous_index = class.to_dbm_index[transi];
                if class.dbm[(previous_index, 0)] < class.dbm[(fired_i, 0)] {
                    return None
                }
                next_dbm[(dbm_index, 0)] = class.dbm[(previous_index, fired_i)];
                next_dbm[(0, dbm_index)] = class.dbm[(fired_i, previous_index)];
            } else if newen.contains(&transi) {
                let dbm_index = from_dbm.len();
                to_dbm[transi] = dbm_index;
                from_dbm.push(transi);
                next_dbm[(dbm_index, 0)] = petri.transitions[transi].interval.1;
                next_dbm[(0, dbm_index)] = -petri.transitions[transi].interval.0;
            } else {
                continue;
            }
        }

        for pers1 in 0..petri.transitions.len() {
            for pers2 in (pers1 + 1)..petri.transitions.len() {
                if (!pers.contains(&pers1)) || (!pers.contains(&pers2)) {
                    continue;
                }
                let prev_index_1 = class.to_dbm_index[pers1];
                let prev_index_2 = class.to_dbm_index[pers2];
                let index_1 = to_dbm[pers1];
                let index_2 = to_dbm[pers2];
                next_dbm[(index_1, index_2)] = class.dbm[(prev_index_1, prev_index_2)];
                next_dbm[(index_2, index_1)] = class.dbm[(prev_index_2, prev_index_1)];
            }
        }

        next_dbm.make_canonical();

        if next_dbm.is_empty() {
            return None;
        }

        Some(StateClass {
            discrete,
            dbm : next_dbm,
            to_dbm_index : to_dbm,
            from_dbm_index : from_dbm,
            predecessors : Vec::new()
        })
    }

    fn make_edges(&mut self) {
        for (i, class) in self.classes.iter().enumerate() {
            for (pred, action) in class.predecessors.iter() {
                let mut edge = Edge::new_weighted(
                    Label::from_string(pred), 
                    Label::from_string(i),
                     *action
                );
                edge.label = Label::from_string(action);
                self.edges.push(edge);
            }
        }
    }

}

impl Model for ClassGraph {

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name : lbl("ClassGraph"),
            description : String::from("Petri net Class graph, each node is associated with a DBM and is an aggregate of possible Petri states"),
            characteristics : TIMED | CONTROLLABLE | SYMBOLIC,
        }
    }

    // Not optimized AT ALL ! Class graph is made for back-propagation
    fn next(&self, state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>) {
        let mut next_index : Option<usize> = None;
        let class_index = state.discrete[state.discrete.nrows() - 1] as usize;
        for e in self.edges.iter() {
            if e.node_from() == class_index && e.weight == action {
                next_index = Some(e.node_to());
            }
        }
        if next_index.is_none() {
            return (None, HashSet::new());
        }
        let next_index = next_index.unwrap();
        let next_class = &self.classes[next_index];
        let mut next_state = next_class.generate_image_state();
        next_state.discrete = next_state.discrete.insert_row(next_class.discrete.nrows(), next_index as i32);
        let actions = self.actions_available(&next_state);
        (Some(next_state), actions)
    }

    fn actions_available(&self, state : &ModelState) -> HashSet<usize> {
        let mut actions = HashSet::new();
        let class_index = state.discrete[state.discrete.nrows() - 1] as usize;
        for e in self.edges.iter() {
            if e.node_from() == class_index {
                actions.insert(e.weight);
            }
        }
        actions
    }

    fn available_delay(&self, _state : &ModelState) -> ClockValue {
        ClockValue::zero()
    }

    fn n_vars(&self) -> usize {
        if self.classes.is_empty() {
            return 0;
        }
        self.classes.first().unwrap().discrete.nrows() + 1 // Additionnal discrete var to remember current class
    }

    fn n_clocks(&self) -> usize {
        self.n_clocks
    }

    fn map_label_to_var(&self, var : Label) -> Option<usize> {
        if !self.places_dic.contains_key(&var) { 
            return None;
        }
        Some(self.places_dic[&var])
    }

}