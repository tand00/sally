mod state_class;
pub use state_class::StateClass;

use core::panic;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::{Rc, Weak};

use num_traits::Zero;

use crate::computation::DBM;

use super::time::ClockValue;
use super::{lbl, new_ptr, ComponentPtr, Edge, Label, Model, ModelMeta, ModelState, CONTROLLABLE, SYMBOLIC, TIMED};
use super::petri::PetriNet;

const CLASS_LIMIT : usize = 4096;

#[derive(Clone)]
pub struct ClassGraph {
    pub classes: Vec<ComponentPtr<StateClass>>,
    pub edges: Vec<Edge<usize, StateClass, StateClass>>,
    pub places_dic : HashMap<Label, usize>,
    pub transitions_count : usize
}

impl ClassGraph {

    pub fn compute(p_net : &PetriNet, initial_state : &ModelState) -> Self {
        let mut cg = ClassGraph {
            classes: Vec::new(),
            edges: Vec::new(),
            places_dic : p_net.places_dic.clone(),
            transitions_count : p_net.transitions.len()
        };
        let mut seen : HashMap<u64, usize> = HashMap::new();
        let mut to_see : VecDeque<usize> = VecDeque::new();
        let initial_class = StateClass::compute_class(p_net, initial_state);
        seen.insert(initial_class.get_hash(), 0);
        cg.classes.push(new_ptr(initial_class));
        to_see.push_back(0);
        while !to_see.is_empty() {
            let class_index = to_see.pop_back().unwrap();
            let class = Rc::clone(&cg.classes[class_index]);
            let clocks = class.borrow().enabled_clocks();
            for action in clocks {
                let next_class = ClassGraph::successor(p_net, &class, action);
                if next_class.is_none() {
                    continue;
                }
                let mut next_class = next_class.unwrap();
                let new_hash = next_class.get_hash();
                if seen.contains_key(&new_hash) {
                    cg.classes[seen[&new_hash]].borrow_mut().predecessors.push((Rc::downgrade(&class), action));
                    continue;
                }
                let new_index = cg.classes.len();
                next_class.index = new_index;
                seen.insert(new_hash, new_index);
                cg.classes.push(new_ptr(next_class));
                to_see.push_back(new_index);
                if cg.classes.len() > CLASS_LIMIT {
                    panic!("Class limit overflow ! Petri net may not be bounded !");
                }
            }
        }
        cg.compile().unwrap(); //TODO! Move compile out of init maybe ?
        cg
    }

    pub fn successor(petri : &PetriNet, class : &ComponentPtr<StateClass>, action : usize) -> Option<StateClass> {
        let image_state = class.borrow().generate_image_state();
        let (next_state, newen, pers) = petri.fire(image_state, action);

        let vars = newen.len() + pers.len();
        let mut next_dbm = DBM::new(vars);
        let mut to_dbm : Vec<usize> = vec![0 ; petri.transitions.len()];
        let mut from_dbm : Vec<usize> = vec![0];
        let prev_to_dbm = &class.borrow().to_dbm_index;
        let fired_i = prev_to_dbm[action];
        let discrete = next_state.discrete;
        let dbm = &class.borrow().dbm;

        for transi in 0..petri.transitions.len() {
            if pers.contains(&transi) {
                let dbm_index = from_dbm.len();
                to_dbm[transi] = dbm_index;
                from_dbm.push(transi);
                let previous_index = prev_to_dbm[transi];
                if dbm[(previous_index, 0)] < dbm[(fired_i, 0)] {
                    return None
                }
                next_dbm[(dbm_index, 0)] = dbm[(previous_index, fired_i)];
                next_dbm[(0, dbm_index)] = dbm[(fired_i, previous_index)];
            } else if newen.contains(&transi) {
                let dbm_index = from_dbm.len();
                to_dbm[transi] = dbm_index;
                from_dbm.push(transi);
                next_dbm[(dbm_index, 0)] = petri.transitions[transi].borrow().interval.1;
                next_dbm[(0, dbm_index)] = -petri.transitions[transi].borrow().interval.0;
            } else {
                continue;
            }
        }

        for pers1 in 0..petri.transitions.len() {
            for pers2 in (pers1 + 1)..petri.transitions.len() {
                if (!pers.contains(&pers1)) || (!pers.contains(&pers2)) {
                    continue;
                }
                let prev_index_1 = prev_to_dbm[pers1];
                let prev_index_2 = prev_to_dbm[pers2];
                let index_1 = to_dbm[pers1];
                let index_2 = to_dbm[pers2];
                next_dbm[(index_1, index_2)] = dbm[(prev_index_1, prev_index_2)];
                next_dbm[(index_2, index_1)] = dbm[(prev_index_2, prev_index_1)];
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
            predecessors : vec![(Rc::downgrade(&class), action)],
            index : 0
        })
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
            if !e.has_source() || !e.has_target() {
                continue;
            }
            if e.ptr_node_from().borrow().index == class_index && e.weight == action {
                next_index = Some(e.ptr_node_to().borrow().index);
            }
        }
        if next_index.is_none() {
            return (None, HashSet::new());
        }
        let next_index = next_index.unwrap();
        let next_class = &self.classes[next_index].borrow();
        let mut next_state = next_class.generate_image_state();
        next_state.discrete = next_state.discrete.insert_row(next_class.discrete.nrows(), next_index as i32);
        let actions = self.available_actions(&next_state);
        (Some(next_state), actions)
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<usize> {
        let mut actions = HashSet::new();
        let class_index = state.discrete[state.discrete.nrows() - 1] as usize;
        for e in self.edges.iter() {
            if !e.has_source() {
                continue;
            }
            if e.ptr_node_from().borrow().index == class_index {
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
        self.classes.first().unwrap().borrow().discrete.nrows() + 1 // Additionnal discrete var to remember current class
    }

    fn map_label_to_var(&self, var : &Label) -> Option<usize> {
        if !self.places_dic.contains_key(var) { 
            return None;
        }
        Some(self.places_dic[var])
    }

    fn init_initial_clocks(&self, mut state : ModelState) -> ModelState {
        if state.discrete.nrows() == 0 {
            return state;
        }
        state.create_clocks(self.transitions_count);
        let current_class = state.discrete[self.n_vars() - 1] as usize;
        let class = Rc::clone(&self.classes[current_class]);
        for t in class.borrow().from_dbm_index.iter().skip(1) {
            state.enable_clock(*t, ClockValue::zero());
        }
        state
    }

    fn is_timed(&self) -> bool {
        false
    }

    fn is_stochastic(&self) -> bool {
        false
    }

    fn compile(&mut self) -> super::CompilationResult<()> {
        self.edges.clear();
        for class in self.classes.iter() {
            for (pred, action) in class.borrow().predecessors.iter() {
                let edge = Edge {
                    label : Label::from(action.to_string()),
                    from : None,
                    to : None,
                    weight : *action,
                    ref_from : Some(Weak::clone(pred)),
                    ref_to : Some(Rc::downgrade(class))
                };
                self.edges.push(edge);
            }
        }
        Ok(())
    }

}