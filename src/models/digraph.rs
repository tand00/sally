use std::{cmp::min, collections::HashSet, ops::Add, rc::Rc};

use nalgebra::{DMatrix, DVector, Scalar};
use num_traits::{Bounded, Zero};

use crate::computation::DBM;

use super::{lbl, new_ptr, node::DataNode, time::{ClockValue, TimeBound}, ComponentPtr, Edge, Label, Model, ModelMeta, ModelState, Node, NONE};

// T is the type to be stored in Nodes, while U is the type of edges weights
pub struct Digraph<T : ToString + 'static, U> {
    pub nodes : Vec<ComponentPtr<DataNode<T, U>>>
}

impl<T : ToString + 'static, U> Digraph<T,U> {

    pub fn new() -> Self {
        Digraph {
            nodes : Vec::new()
        }
    }

    pub fn make_node(&mut self, value : T) {
        let node = new_ptr(DataNode::from(value));
        node.borrow_mut().index = self.nodes.len();
        self.nodes.push(node);
    }

    pub fn from(data : Vec<T>) -> Self 
    where T : Clone 
    {
        let nodes : Vec<ComponentPtr<DataNode<T, U>>> = data.iter().enumerate().map(|(i,x)| {
            let node = new_ptr(DataNode::from(x.clone()));
            node.borrow_mut().index = i;
            node
        }).collect();
        Digraph { nodes }
    }

    pub fn make_edge(&mut self, from : T, to : T, weight : U) 
    where T : PartialEq
    {
        let mut e = Edge::new_weighted(
            Label::new(), 
            Label::new(), 
            weight);
        let mut node_from : Option<ComponentPtr<DataNode<T,U>>> = None;
        let mut node_to : Option<ComponentPtr<DataNode<T,U>>> = None;
        for node in self.nodes.iter() {
            if node.borrow().element == from {
                e.set_node_from(node);
                e.from = Some(node.borrow().get_label());
                node_from = Some(Rc::clone(node));
            }
            if node.borrow().element == to {
                e.set_node_to(node);
                e.from = Some(node.borrow().get_label());
                node_to = Some(Rc::clone(node));
            }
        }
        let e = Rc::new(e);
        if node_from.is_some() {
            node_from.unwrap().borrow_mut().out_edges.push(Rc::clone(&e));
        }
        if node_to.is_some() {
            node_to.unwrap().borrow_mut().in_edges.push(Rc::clone(&e));
        }
    }

    // Implementation of the Floyd-Warshall algorithm
    pub fn shortest_paths(&self) -> DMatrix<U> 
    where U : Add<Output = U> + Ord + Zero + Bounded + Scalar
    {
        let n_nodes = self.nodes.len();
        let mut distances = 
            DMatrix::from_fn(n_nodes, n_nodes, |i,j| {
                if i == j { U::zero() }
                else { U::max_value() }
            });
        for (i,node) in self.nodes.iter().enumerate() {
            for edge in node.borrow().out_edges.iter() {
                if !edge.has_target() {
                    continue;
                }
                let j = edge.ptr_node_to().borrow().index;
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
    where U : Scalar + Zero + Bounded + PartialOrd
    {
        let n_nodes = self.nodes.len();
        let mut res = DMatrix::from_fn(n_nodes, n_nodes, |i,j| {
            if i == j { U::zero() }
            else { U::max_value() }
        });
        for (i, node) in self.nodes.iter().enumerate() {
            for edge in node.borrow().out_edges.iter() {
                if !edge.has_target() {
                    continue;
                }
                let j = edge.ptr_node_to().borrow().index;
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
                from.borrow().get_label(), 
                to.borrow().get_label(), 
                w.clone());
            e.set_node_from(from);
            e.set_node_to(to);
            let e = Rc::new(e);
            from.borrow_mut().out_edges.push(Rc::clone(&e));
            from.borrow_mut().in_edges.push(Rc::clone(&e));
        }
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

impl<T : 'static + ToString, U : 'static> Model for Digraph<T,U> {

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name : lbl("Digraph"),
            description : String::from("Simple generic directed graph, whose types can be set to anything"),
            characteristics : NONE
        }
    }

    fn next(&self, mut state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>) {
        let (index, _) = state.discrete.argmax();
        let node = &self.nodes[index];
        let edge = Rc::clone(&node.borrow().out_edges[action]);
        if edge.node_to().is_none() {
            return (None, HashSet::new())
        }
        let next_index = edge.ptr_node_to().borrow().index;
        state.discrete[index] = 0;
        state.discrete[next_index] = 1;
        let len_actions = edge.ptr_node_to().borrow().out_edges.len();
        let actions : HashSet<usize> = (0..len_actions).collect();
        if actions.is_empty() {
            state.deadlocked = true;
        }
        (Some(state), actions)
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<usize> {
        let (index, _) = state.discrete.argmax();
        let len_actions = self.nodes[index].borrow().out_edges.len();
        (0..len_actions).collect()
    }

    fn n_vars(&self) -> usize {
        self.nodes.len()
    }

    fn map_label_to_var(&self, var : &Label) -> Option<usize> {
        for (i,n) in self.nodes.iter().enumerate() {
            if n.borrow().get_label() == *var {
                return Some(i);
            }
        }
        return None;
    }

    fn is_timed(&self) -> bool {
        false
    }

    fn is_stochastic(&self) -> bool {
        false
    }

}