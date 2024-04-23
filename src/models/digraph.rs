use std::{cmp::min, collections::HashSet, ops::Add};

use nalgebra::{DMatrix, DVector, Scalar};
use num_traits::{Bounded, Zero};

use crate::computation::DBM;

use super::{lbl, node::SimpleNode, time::{ClockValue, TimeBound}, Edge, Label, Model, ModelMeta, ModelState, Node, NONE};

// T is the type to be stored in Nodes, while U is the type of edges weights
pub struct Digraph<T,U> {
    pub edges : Vec<Edge<U>>,
    pub nodes : Vec<SimpleNode<T>>
}

impl<T,U> Digraph<T,U> {

    pub fn new() -> Self {
        Digraph {
            edges : Vec::new(),
            nodes : Vec::new()
        }
    }

    pub fn make_node(&mut self, value : T) {
        let node = SimpleNode::from(value);
        self.nodes.push(node);
    }

    pub fn from(data : Vec<T>) -> Self 
    where T : Clone 
    {
        let nodes : Vec<SimpleNode<T>> = data.iter().map(|x| SimpleNode::from(x.clone())).collect();
        Digraph {
            nodes,
            edges : Vec::new()
        }
    }

    pub fn make_edge(&mut self, from : T, to : T, weight : U) 
    where T : ToString + PartialEq
    {
        let mut e = Edge::new_weighted(
            Label::new(), 
            Label::new(), 
            weight);
        for (i, n) in self.nodes.iter_mut().enumerate() {
            if n.element == from {
                e.set_node_from(i);
                e.from = Some(n.get_label());
                n.out_edges.push(self.edges.len());
            }
            if n.element == to {
                e.set_node_to(i);
                e.to = Some(n.get_label());
                n.in_edges.push(self.edges.len());
            }

        }
        self.edges.push(e);
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
        for i in 0..n_nodes {
            let node = &self.nodes[i];
            for edge_i in node.out_edges.iter() {
                let j = self.edges[*edge_i].node_to();
                let weight = self.edges[*edge_i].weight.clone();
                distances[(i,j)] = weight;
            }
        }
        for k in 0..n_nodes {
            for i in 0..n_nodes {
                for j in 0..n_nodes {
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
        T : ToString + Clone,
        U : Add<Output = U> + Ord + Zero + Bounded + Scalar
    {
        let distances = self.shortest_paths();
        let mut res = Digraph {
            nodes : self.nodes.clone(),
            edges : Vec::new()
        };
        res.create_relations(distances);
        res
    }

    pub fn from_matrix(elements : Vec<T>, relations : DMatrix<U>) -> Self 
    where
        T : ToString + Clone,
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
        for (i, n) in self.nodes.iter().enumerate() {
            for e_i in n.out_edges.iter() {
                let edge = &self.edges[*e_i];
                let j = edge.node_to();
                res[(i,j)] = edge.weight.clone();
            }
        }
        res
    }

    pub fn create_relations(&mut self, relations : DMatrix<U>) 
    where
        T : ToString + Clone,
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
            e.set_node_from(i);
            e.set_node_to(j);
            self.edges.push(e);
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

    fn next(&self, state : ModelState, action : usize) -> (Option<ModelState>, HashSet<usize>) {
        if state.discrete.nrows() == 0 {
            return (None, HashSet::new());
        }
        let index = state.discrete.argmax().0;
        let from_node = &self.nodes[index];
        if from_node.out_edges.contains(&action) {
            return (None, HashSet::new());
        }
        let edge = &self.edges[action];
        let node_to_index = edge.node_to();
        let node_to = &self.nodes[node_to_index];
        let next_discrete : DVector<i32> = DVector::from_fn(self.n_vars(), |i, _j| {
            if i == node_to_index { 1 } else { 0 }
        });
        let next_actions = node_to.out_edges.iter().map(|i| *i).collect();
        (Some(ModelState {
            discrete : next_discrete,
            clocks : DVector::from_element(0, ClockValue::disabled()),
            deadlocked : node_to.out_edges.len() == 0
        }), next_actions)
    }

    fn actions_available(&self, state : &ModelState) -> HashSet<usize> {
        if state.discrete.nrows() == 0 {
            return HashSet::new();
        }
        let index = state.discrete.argmax().0;
        self.nodes[index].out_edges.iter().map(|i| *i).collect()
    }

    fn n_vars(&self) -> usize {
        self.nodes.len()
    }

    fn map_label_to_var(&self, var : Label) -> Option<usize> {
        for (i,n) in self.nodes.iter().enumerate() {
            if n.get_label() == var {
                return Some(i);
            }
        }
        return None;
    }

}