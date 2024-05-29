use std::{cmp::min, ops::Add, sync::Arc};

use nalgebra::{DMatrix, Scalar};
use num_traits::{Bounded, Zero};

use crate::computation::DBM;

use super::{node::DataNode, time::TimeBound, Edge, Label, Node};

// T is the type to be stored in Nodes, while U is the type of edges weights
pub struct Digraph<T : ToString + 'static, U> {
    pub nodes : Vec<Arc<DataNode<T, U>>>,
    pub edges : Vec<Arc<Edge<U, DataNode<T, U>, DataNode<T, U>>>>,
}

impl<T : ToString, U> Digraph<T,U> {

    pub fn new() -> Self {
        Self {
            nodes : Vec::new(),
            edges : Vec::new(),
        }
    }

    pub fn make_node(&mut self, value : T) {
        let mut node = DataNode::from(value);
        node.index = self.nodes.len();
        self.nodes.push(Arc::new(node));
    }

    pub fn from(data : Vec<T>) -> Self 
    where 
        T : Clone 
    {
        let nodes : Vec<Arc<DataNode<T, U>>> = data.iter().enumerate().map(|(i,x)| {
            let mut node = DataNode::from(x.clone());
            node.index = i;
            Arc::new(node)
        }).collect();
        Digraph { nodes, ..Default::default() }
    }

    pub fn make_edge(&mut self, from : T, to : T, weight : U) 
    where 
        T : PartialEq
    {
        let mut e = Edge::new_weighted(
            Label::new(), 
            Label::new(), 
            weight);
        let mut node_from : Option<Arc<DataNode<T,U>>> = None;
        let mut node_to : Option<Arc<DataNode<T,U>>> = None;
        for node in self.nodes.iter() {
            if node.element == from {
                e.set_node_from(node);
                e.from = Some(node.get_label());
                node_from = Some(Arc::clone(node));
            }
            if node.element == to {
                e.set_node_to(node);
                e.from = Some(node.get_label());
                node_to = Some(Arc::clone(node));
            }
        }
        e.label = e.from.clone().unwrap_or_default() + "->" + e.to.clone().unwrap_or_default();
        let e = Arc::new(e);
        if node_from.is_some() {
            node_from.unwrap().out_edges.write().unwrap().push(Arc::clone(&e));
        }
        if node_to.is_some() {
            node_to.unwrap().out_edges.write().unwrap().push(Arc::clone(&e));
        }
        self.edges.push(e);
    }

    // Implementation of the Floyd-Warshall algorithm
    pub fn shortest_paths(&self) -> DMatrix<U> 
    where 
        U : Add<Output = U> + Ord + Zero + Bounded + Scalar
    {
        let n_nodes = self.nodes.len();
        let mut distances = 
            DMatrix::from_fn(n_nodes, n_nodes, |i,j| {
                if i == j { U::zero() }
                else { U::max_value() }
            });
        for (i,node) in self.nodes.iter().enumerate() {
            for edge in node.out_edges.read().unwrap().iter() {
                if !edge.has_target() {
                    continue;
                }
                let j = edge.get_node_to().index;
                distances[(i,j)] = edge.weight.clone();
            }
        }
        for k in 0..n_nodes {
            for i in 0..n_nodes {
                for j in 0..n_nodes {
                    if distances[(i,k)] == U::max_value() || distances[(k,j)] == U::max_value() {
                        continue;
                    }
                    distances[(i,j)] = min(
                        distances[(i,j)].clone(),
                        distances[(i,k)].clone() + distances[(k,j)].clone()
                    );
                }
            }
        }
        distances
    }

    pub fn shortest_digraph(&self) -> Self 
    where 
        T : Clone,
        U : Add<Output = U> + Ord + Zero + Bounded + Scalar
    {
        let distances = self.shortest_paths();
        let mut res = Digraph {
            nodes : self.nodes.clone(),
            ..Default::default()
        };
        res.create_relations(distances);
        res
    }

    pub fn from_matrix(elements : Vec<T>, relations : DMatrix<U>) -> Self 
    where
        T : Clone,
        U : Ord + Clone + Bounded + Zero
    {
        let mut graph = Self::from(elements);
        graph.create_relations(relations);
        graph
    }

    pub fn to_matrix(&self) -> DMatrix<U> 
    where 
        U : Scalar + Zero + Bounded + PartialOrd
    {
        let n_nodes = self.nodes.len();
        let mut res = DMatrix::from_fn(n_nodes, n_nodes, |i,j| {
            if i == j { U::zero() }
            else { U::max_value() }
        });
        for (i, node) in self.nodes.iter().enumerate() {
            for edge in node.out_edges.read().unwrap().iter() {
                if !edge.has_target() {
                    continue;
                }
                let j = edge.get_node_to().index;
                res[(i,j)] = edge.weight.clone();
            }
        }
        res
    }

    pub fn create_relations(&mut self, relations : DMatrix<U>) 
    where
        T : Clone,
        U : Ord + Clone + Bounded + Zero
    {
        let n_nodes = self.nodes.len();
        for (n, w) in relations.iter().enumerate() {
            let i = n / n_nodes;
            let j = n % n_nodes;
            if *w >= U::max_value() || (w.is_zero() && i == j) { // Max length = INF, min length to self = no edge
                continue;
            }
            let from = &self.nodes[i];
            let to = &self.nodes[j];
            let mut e = Edge::new_weighted(
                from.get_label(), 
                to.get_label(), 
                w.clone());
            e.set_node_from(from);
            e.set_node_to(to);
            let e = Arc::new(e);
            from.out_edges.write().unwrap().push(Arc::clone(&e));
            from.in_edges.write().unwrap().push(Arc::clone(&e));
        }
    }

}

impl<T : ToString, U> Default for Digraph<T,U> {
    fn default() -> Self {
        Digraph::new()
    }
}

impl Digraph<usize, TimeBound> {

    pub fn from_dbm(matrix : DBM) -> Self {
        let mut graph = Digraph::new();
        for i in 0..(matrix.vars_count() + 1) {
            graph.make_node(i);
        }
        for i in 0..(matrix.vars_count() + 1) {
            for j in 0..(matrix.vars_count() + 1) {
                graph.make_edge(i, j, matrix[(i,j)]);
            }
        }
        graph
    }

    pub fn to_dbm(&self) -> DBM {
        DBM::from(self.to_matrix())
    }

}

// I don't think that Digraph must implement Model, it's better to use it as a data structure in a model wrapper (ex : markov chain)
/*impl<T : ToString, U> Model for Digraph<T,U> {

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name : lbl("Digraph"),
            description : String::from("Simple generic directed graph, whose types can be set to anything"),
            characteristics : NONE
        }
    }

    fn next(&self, mut state : ModelState, action : Action) -> (Option<ModelState>, HashSet<Action>) {
        let vars_refs : Vec<Ref<DataNode<T,U>>> = self.nodes.iter().map(|n| n.borrow() ).collect();
        let vars = vars_refs.iter().map(|r| r.get_var() );
        let index = state.argmax(vars);
        let node = Rc::clone(&self.nodes[index]);
        let edge = Rc::clone(&node.borrow().out_edges[0]);
        if edge.node_to().is_none() {
            return (None, HashSet::new())
        }
        let next_node = edge.ptr_node_to();
        state.unmark(node.borrow().get_var(), 1);
        state.mark(next_node.borrow().get_var(), 1);
        let actions : HashSet<Action> = self.available_actions(&state);
        if actions.is_empty() {
            state.deadlocked = true;
        }
        (Some(state), actions)
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<Action> {
        let mut available : HashSet<Action> = HashSet::new();
        let vars_refs : Vec<Ref<DataNode<T,U>>> = self.nodes.iter().map(|n| n.borrow() ).collect();
        let vars = vars_refs.iter().map(|r| r.get_var() );
        let index = state.argmax(vars);
        let node = Rc::clone(&self.nodes[index]);
        let this_index = node.borrow().index;
        for edge in node.borrow().out_edges.iter() {
            if !edge.has_target() {
                continue;
            }
            let target = edge.ptr_node_to();
            let target_index = target.borrow().index;
            let action = self.actions[&(this_index, target_index)];
            available.insert(action);
        }
        available
    }

    fn is_timed(&self) -> bool {
        false
    }

    fn is_stochastic(&self) -> bool {
        false
    }

    fn compile(&mut self, context : &mut ModelContext) -> CompilationResult<()> {
        for node in self.nodes.iter() {
            node.borrow_mut().make_var(context, VarType::VarI8);
            let this_label = node.borrow().get_label();
            let this_index = node.borrow().index;
            for edge in node.borrow().out_edges.iter() { 
                let target_label = if !edge.has_target() {
                    Label::new()
                } else {
                    let target = edge.ptr_node_to();
                    target.borrow().get_label()
                };
                let action_label = this_label.clone() + "_" + target_label;
                let action = context.add_action(action_label);
                edge
            }
        }
        Ok(())
    }

}*/