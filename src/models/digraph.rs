use std::{cmp::min, marker::PhantomData, ops::Add, sync::Arc};

use nalgebra::{DMatrix, Scalar};
use num_traits::{Bounded, Zero};

use crate::computation::DBM;

use super::{node::{Node, DataNode}, time::TimeBound, Edge, Label};

// T is the type to be stored in Nodes, while U is the type of edges weights
pub struct Digraph<T : 'static, U> {
    pub nodes : Vec<Arc<DataNode<T, U>>>,
    pub edges : Vec<Arc<Edge<U, DataNode<T, U>, DataNode<T, U>>>>,
}

impl<T, U> Digraph<T,U> {

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
                node_from = Some(Arc::clone(node));
            }
            if node.element == to {
                e.set_node_to(node);
                node_to = Some(Arc::clone(node));
            }
        }
        let e = Arc::new(e);
        if let Some(node_from) = node_from {
            node_from.add_out_edge(&e);
        }
        if let Some(node_to) = node_to {
            node_to.add_in_edge(&e);
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
                let e = Edge::data_edge(from, to, weight.clone());
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
        self.floyd_warshall(U::clone, U::max_value())
    }

    pub fn shortest_float_paths<F>(&self, weight : F) -> DMatrix<f64> 
    where 
        F : Fn(&U) -> f64
    {
        self.floyd_warshall(weight, f64::INFINITY)
    }

    pub fn floyd_warshall<F,V>(&self, weight : F, no_edge : V) -> DMatrix<V> 
    where 
        F : Fn(&U) -> V,
        V : Add<Output = V> + PartialOrd + Zero + Scalar
    {
        let n_nodes = self.nodes.len();
        let mut distances = self.make_weight_matrix(weight, no_edge.clone());
        for k in 0..n_nodes {
            for i in 0..n_nodes {
                for j in 0..n_nodes {
                    if distances[(i,k)] >= no_edge || distances[(k,j)] >= no_edge {
                        continue;
                    }
                    let sum = distances[(i,k)].clone() + distances[(k,j)].clone();
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

    pub fn dijkstra<F,V>(
        &self, from : &Arc<DataNode<T,U>>, to : &Arc<DataNode<T,U>>, 
        weight : F, no_edge : V
    ) -> (V, Vec<Arc<Edge<T,DataNode<T,U>, DataNode<T,U>>>>)
    where
        F : Fn(&U) -> V,
        V : Add<Output = V> + PartialOrd + Zero + Scalar
    {
        //let mut seqs = Vec::new();
        let mut dists = vec![no_edge.clone() ; self.nodes.len()];
        let matrix = self.make_weight_matrix(weight, no_edge);
        todo!()
    }

    pub fn from_matrix(elements : Vec<T>, relations : DMatrix<U>) -> Self 
    where
        U : Ord + Clone + Bounded + Zero
    {
        let mut graph = Self::from(elements);
        graph.create_relations(relations);
        graph
    }

    pub fn get_matrix(&self) -> DMatrix<U> 
    where 
        U : Scalar + Zero + Bounded
    {
        self.make_weight_matrix(U::clone, U::max_value())
    }

    pub fn get_float_matrix<F>(&self, weight : F) -> DMatrix<f64> 
    where 
        F : Fn(&U) -> f64
    {
        self.make_weight_matrix(weight, f64::INFINITY)
    }

    pub fn make_weight_matrix<F,V>(&self, weight : F, no_edge : V) -> DMatrix<V> 
    where 
        F : Fn(&U) -> V,
        V : Scalar + Zero
    {
        let n_nodes = self.nodes.len();
        let mut res = DMatrix::from_fn(n_nodes, n_nodes, |i,j| {
            if i == j { V::zero() }
            else { no_edge.clone() }
        });
        for (i, node) in self.nodes.iter().enumerate() {
            for edge in node.out_edges.read().unwrap().iter() {
                if !edge.has_target() {
                    continue;
                }
                let j = edge.get_node_to().index;
                res[(i,j)] = weight(&edge.weight);
            }
        }
        res
    }

    pub fn make_edges_matrix(&self) 
        -> Vec<Vec<Option<Arc<Edge<U,DataNode<T,U>,DataNode<T,U>>>>>>
    {
        let n_nodes = self.nodes.len();
        let res = vec![None ; n_nodes];
        let mut res = vec![res ; n_nodes];
        for (i, node) in self.nodes.iter().enumerate() {
            for edge in node.out_edges.read().unwrap().iter() {
                if !edge.has_target() {
                    continue;
                }
                let j = edge.get_node_to().index;
                res[i][j] = Some(Arc::clone(&edge));
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
            let e = Edge::data_edge(from, to, w.clone());
            let e = Arc::new(e);
            from.add_out_edge(&e);
            to.add_in_edge(&e);
        }
    }

}

impl<T, U> From<Vec<T>> for Digraph<T,U> {
    fn from(data : Vec<T>) -> Self {
        let nodes : Vec<Arc<DataNode<T, U>>> = data.into_iter().enumerate().map(|(i,x)| {
            let mut node = DataNode::from(x);
            node.index = i;
            Arc::new(node)
        }).collect();
        Digraph { nodes, ..Default::default() }
    }
}

impl<T : ToString, U> Digraph<T,U> {

    pub fn labelize(&mut self) 
        where T : ToString, U : Clone
    {
        for node in self.nodes.iter() {
            node.clear_edges();
        }
        let mut new_edges = Vec::new();
        for edge in self.edges.iter() {
            let mut new_edge = Edge::orphan(edge.weight.clone());
            if edge.has_source() {
                let source = edge.get_node_from();
                new_edge.set_node_from(&source);
                new_edge.from = Some(source.get_label());                
            }
            if edge.has_target() {
                let target = edge.get_node_to();
                new_edge.set_node_to(&target);
                new_edge.to = Some(target.get_label());
            }
            let new_edge = Arc::new(new_edge);
            if new_edge.has_source() {
                new_edge.get_node_from().add_out_edge(&new_edge);
            }
            if new_edge.has_target() {
                new_edge.get_node_to().add_in_edge(&new_edge);
            }
            new_edges.push(new_edge);
        }
        self.edges = new_edges;
    }

}

impl<T, U> Default for Digraph<T,U> {
    fn default() -> Self {
        Digraph::new()
    }
}

impl From<DBM> for Digraph<usize, TimeBound> {
    fn from(matrix: DBM) -> Self {
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
}

impl From<Digraph<usize, TimeBound>> for DBM {
    fn from(graph: Digraph<usize, TimeBound>) -> Self {
        DBM::from(graph.get_matrix())
    }
}