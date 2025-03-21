mod state_class;
pub use state_class::StateClass;

use core::panic;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock, Weak};

use num_traits::Zero;

use crate::computation::convex::Convex;
use crate::computation::virtual_memory::EvaluationType;
use crate::computation::DBM;
use crate::verification::smc::RandomRunIterator;
use crate::verification::{Verifiable, VerificationBound};

use super::action::Action;
use super::digraph::search_strategy::{GraphTraversal, NeighborsFinder, SearchStrategy};
use super::model_context::ModelContext;
use super::model_var::{ModelVar, VarType};
use super::time::{Bound, ClockValue};
use super::{lbl, Edge, Label, Model, ModelMeta, ModelState, Node, CONTROLLABLE, SYMBOLIC, TIMED, UNMAPPED_ID};
use super::petri::{PetriNet, PetriTransition};

const CLASS_LIMIT : usize = u16::MAX as usize;

#[derive(Clone)]
pub struct ClassGraph {
    pub id : usize,
    pub classes : Vec<Arc<StateClass>>,
    pub edges : Vec<Edge<Action, StateClass, StateClass>>,
    pub current_class : ModelVar,
    pub transitions : Vec<Arc<PetriTransition>>
}

impl ClassGraph {

    pub fn compute(p_net : &PetriNet, initial_state : &ModelState) -> Self {
        let mut cg = ClassGraph {
            id : UNMAPPED_ID,
            classes : Vec::new(),
            edges : Vec::new(),
            current_class : ModelVar::name(lbl("CurrentClass")),
            transitions : p_net.transitions.clone()
        };
        cg.current_class.set_type(VarType::VarU16);
        let mut seen : HashMap<u64, usize> = HashMap::new();
        let mut to_see : VecDeque<usize> = VecDeque::new();
        let initial_class = StateClass::compute_class(p_net, initial_state);
        seen.insert(initial_class.get_hash(), 0);
        cg.classes.push(Arc::new(initial_class));
        to_see.push_back(0);
        while !to_see.is_empty() {
            let class_index = to_see.pop_back().unwrap();
            let class = Arc::clone(&cg.classes[class_index]);
            let clocks = class.enabled_clocks();
            for t_index in clocks {
                let next_class = ClassGraph::successor(p_net, &class, t_index);
                let action = cg.transitions[t_index].get_action();
                let Some(mut next_class) = next_class else {
                    continue;
                };
                let new_hash = next_class.get_hash();
                if seen.contains_key(&new_hash) {
                    cg.classes[seen[&new_hash]].predecessors.write().unwrap().push((Arc::downgrade(&class), action));
                    continue;
                }
                let new_index = cg.classes.len();
                next_class.index = new_index;
                seen.insert(new_hash, new_index);
                cg.classes.push(Arc::new(next_class));
                to_see.push_back(new_index);
                if cg.classes.len() > CLASS_LIMIT {
                    panic!("Class limit overflow ! Petri net may not be bounded !");
                }
            }
        }
        cg
    }

    pub fn successor(petri : &PetriNet, class : &Arc<StateClass>, t_index : usize) -> Option<StateClass> {
        let image_state = class.generate_image_state();
        let (next_state, newen, pers) = petri.fire(image_state, t_index);

        let vars = newen.len() + pers.len();
        let mut next_dbm = DBM::new(vars);
        let mut to_dbm : Vec<usize> = vec![0 ; petri.transitions.len()];
        let mut from_dbm : Vec<usize> = vec![0];
        let prev_to_dbm = &class.to_dbm_index;
        let fired_i = prev_to_dbm[t_index];
        let discrete = next_state.discrete;
        let dbm = &class.dbm;
        let action = petri.get_transition_action(t_index);

        for transi in 0..petri.transitions.len() {
            if pers.contains(&transi) {
                let dbm_index = from_dbm.len();
                to_dbm[transi] = dbm_index;
                from_dbm.push(transi);
                let previous_index = prev_to_dbm[transi];
                if dbm[(previous_index, 0)] < -dbm[(0, fired_i)] {
                    return None;
                }
                next_dbm[(dbm_index, 0)] = std::cmp::min(
                    dbm[(previous_index, 0)] + dbm[(0, fired_i)],
                    dbm[(previous_index, fired_i)]
                );
                next_dbm[(0, dbm_index)] = std::cmp::min(
                    dbm[(fired_i, previous_index)],
                    std::cmp::min(
                        dbm[(0, previous_index)] + dbm[(fired_i, 0)],
                        Bound::Large(0) // useless I think
                    )
                );
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
            predecessors : RwLock::new(vec![(Arc::downgrade(&class), action)]),
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
    fn next(&self, state : ModelState, action : Action) -> Option<ModelState> {
        let mut next_index : Option<usize> = None;
        let class_index = state.evaluate_var(&self.current_class) as usize;
        for e in self.edges.iter() {
            if !e.has_source() || !e.has_target() {
                continue;
            }
            if e.get_node_from().index == class_index && e.weight == action {
                next_index = Some(e.get_node_to().index);
            }
        }
        let Some(next_index) = next_index else {
            return None;
        };
        let next_class = &self.classes[next_index];
        let mut next_state = next_class.generate_image_state();
        next_state.discrete.size_delta(self.current_class.size());
        next_state.discrete.set(&self.current_class, next_index as EvaluationType);
        Some(next_state)
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<Action> {
        let mut actions = HashSet::new();
        let class_index = state.evaluate_var(&self.current_class) as usize;
        for e in self.edges.iter() {
            if !e.has_source() {
                continue;
            }
            if e.get_node_from().index == class_index {
                actions.insert(e.weight.clone());
            }
        }
        actions
    }

    fn init_initial_clocks(&self, mut state : ModelState) -> ModelState {
        let current_class = state.evaluate_var(&self.current_class) as usize;
        let class = &self.classes[current_class];
        for t in class.from_dbm_index.iter().skip(1) {
            let transi = &self.transitions[*t];
            let clock = transi.get_clock();
            state.enable_clock(clock, ClockValue::zero());
        }
        state
    }

    fn is_timed(&self) -> bool {
        false
    }

    fn is_stochastic(&self) -> bool {
        false
    }

    fn compile(&mut self, context : &mut ModelContext) -> super::CompilationResult<()> {
        self.id = context.new_model();
        self.edges.clear();
        for class in self.classes.iter() {
            for (pred, action) in class.predecessors.read().unwrap().iter() {
                let edge = Edge {
                    from : None,
                    to : None,
                    weight : action.clone(),
                    ref_from : Some(Weak::clone(pred)),
                    ref_to : Some(Arc::downgrade(class)),
                };
                self.edges.push(edge);
            }
        }
        self.current_class = context.add_var(self.current_class.name.clone(), self.current_class.get_type());
        Ok(())
    }

    fn random_run<'a>(&'a self, initial : &'a ModelState, bound : VerificationBound)
        -> Box<dyn Iterator<Item = (std::rc::Rc<ModelState>, ClockValue, Option<Action>)> + 'a>
    {
        Box::new(RandomRunIterator::generate(self, initial, bound))
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn nodes_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn super::Node> + 'a> {
        let iter = self.classes.iter().map(|c| c.as_node());
        Box::new(iter)
    }

    fn edges(&self) -> Vec<Edge<String, Label, Label>> {
        self.edges.iter().map(Edge::stringify).collect()
    }

}

pub struct StateClassGenerator<'a> {
    net : &'a PetriNet,
    seen : HashSet<u64>
}

impl<'a> StateClassGenerator<'a> {

    pub fn classes<S : SearchStrategy<Arc<StateClass>>>(strategy : S, net : &'a PetriNet, initial_state : &ModelState)
        -> GraphTraversal<Arc<StateClass>, S, Self>
    {
        let mut gen = Self::from(net);
        let initial = Arc::new(StateClass::compute_class(net, initial_state));
        let hash = initial.get_hash();
        gen.seen.insert(hash);
        GraphTraversal::new(initial, strategy, gen)
    }

}

impl<'a> From<&'a PetriNet> for StateClassGenerator<'a> {

    fn from(net: &'a PetriNet) -> Self {
        StateClassGenerator { net, seen : HashSet::new() }
    }

}

impl<'a> NeighborsFinder<Arc<StateClass>> for StateClassGenerator<'a> {

    fn neighbors(&mut self, x : &Arc<StateClass>) -> Vec<Arc<StateClass>> {
        let clocks = x.enabled_clocks();
        let mut found = vec![];
        for t_index in clocks {
            let next_class = ClassGraph::successor(&self.net, x, t_index);
            let Some(next_class) = next_class else {
                continue;
            };
            let new_hash = next_class.get_hash();
            if self.seen.contains(&new_hash) {
                continue;
            }
            self.seen.insert(new_hash);
            found.push(Arc::new(next_class));
        }
        found
    }

}
