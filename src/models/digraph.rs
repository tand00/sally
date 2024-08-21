use std::{cmp::min, ops::Add, sync::Arc};

use nalgebra::{DMatrix, Scalar};
use num_traits::{Bounded, Zero};

use crate::computation::DBM;

use super::{node::DataNode, time::TimeBound, Edge, Label, Node};

// T is the type to be stored in Nodes, while U is the type of edges weights
pub struct Digraph<T : 'static, U> {
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

    pub fn make_node(&mut self, value : T) -> Arc<DataNode<T,U>> {
        let mut node = DataNode::from(value);
        node.index = self.nodes.len();
        let node = Arc::new(node);
        self.nodes.push(Arc::clone(&node));
        node
    }

    pub fn make_edge(&mut self, from : T, to : T, weight : U) -> Arc<Edge<U, DataNode<T, U>, DataNode<T, U>>>
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
        let e = Arc::new(e);
        if let Some(node_from) = node_from {
            node_from.out_edges.write().unwrap().push(Arc::clone(&e));
        }
        if let Some(node_to) = node_to {
            node_to.in_edges.write().unwrap().push(Arc::clone(&e));
        }
        self.edges.push(Arc::clone(&e));
        e
    }

    pub fn make_edge_when<F>(&mut self, filter : F, weight : U) -> Vec<Arc<Edge<U, DataNode<T, U>, DataNode<T, U>>>>
    where 
        F : Fn (&T,&T) -> bool,
        U : Clone
    {
        let mut res = Vec::new();
        for from in self.nodes.iter() {
            for to in self.nodes.iter() {
                if !filter(&from.element, &to.element) { continue };
                let mut e = Edge::data_edge(from, to, weight.clone());
                e.from = Some(from.get_label());
                e.to = Some(to.get_label());
                let e = Arc::new(e);
                self.edges.push(Arc::clone(&e));
                res.push(e);
            }
        }
        res
    }

    pub fn find<F>(&self, filter : F) -> Vec<Arc<DataNode<T,U>>> 
        where F : Fn(&T) -> bool
    {
        let mut res = Vec::new();
        for node in self.nodes.iter() {
            if filter(&node.element) {
                res.push(Arc::clone(node));
            }
        }
        res
    }

    pub fn find_first<F>(&self, filter : F) -> Option<Arc<DataNode<T,U>>>
        where F : Fn(&T) -> bool
    {
        for node in self.nodes.iter() {
            if filter(&node.element) {
                return Some(Arc::clone(node));
            }
        }
        None
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

    pub fn floyd_warshall<F>(&self, weight : F) -> DMatrix<f64> 
    where 
        F : Fn(&U) -> f64
    {
        let n_nodes = self.nodes.len();
        let mut distances = 
            DMatrix::from_fn(n_nodes, n_nodes, |i,j| {
                if i == j { 0.0 }
                else { f64::INFINITY }
            });
        for (i,node) in self.nodes.iter().enumerate() {
            for edge in node.out_edges.read().unwrap().iter() {
                if !edge.has_target() {
                    continue;
                }
                let j = edge.get_node_to().index;
                distances[(i,j)] = weight(&edge.weight);
            }
        }
        for k in 0..n_nodes {
            for i in 0..n_nodes {
                for j in 0..n_nodes {
                    if distances[(i,k)] == f64::INFINITY || distances[(k,j)] == f64::INFINITY {
                        continue;
                    }
                    let sum = distances[(i,k)] + distances[(k,j)];
                    if sum < distances[(i,j)] {
                        distances[(i,j)] = sum;
                    }
                }
            }
        }
        distances
    }

    pub fn shortest_digraph(&self) -> Self 
    where 
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

impl<T : ToString, U> From<Vec<T>> for Digraph<T,U> {
    fn from(data : Vec<T>) -> Self {
        let nodes : Vec<Arc<DataNode<T, U>>> = data.into_iter().enumerate().map(|(i,x)| {
            let mut node = DataNode::from(x);
            node.index = i;
            Arc::new(node)
        }).collect();
        Digraph { nodes, ..Default::default() }
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