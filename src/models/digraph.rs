use std::{collections::BTreeMap, ops::Add, sync::Arc, usize};

use nalgebra::{DMatrix, Scalar};
use num_traits::{Bounded, Zero};
use search_strategy::GraphTraversal;

use crate::computation::DBM;

use super::{node::{Node, DataNode}, time::TimeBound, Edge};

// T is the type to be stored in Nodes, while U is the type of edges weights

pub type GraphNode<T,U> = Arc<DataNode<T,U>>;
pub type GraphEdge<T,U> = Arc<Edge<U, DataNode<T,U>, DataNode<T,U>>>;

pub mod search_strategy;

pub struct Digraph<T, U> {
    nodes : Vec<GraphNode<T,U>>,
    edges : BTreeMap<(usize, usize), GraphEdge<T,U>>,
}

impl<T, U> Digraph<T,U> {

    pub fn new() -> Self {
        Self {
            nodes : Vec::new(),
            edges : Default::default(),
        }
    }

    pub fn make_node(&mut self, value : T) -> GraphNode<T,U> {
        let mut node = DataNode::from(value);
        node.index = self.nodes.len();
        let node = Arc::new(node);
        self.nodes.push(Arc::clone(&node));
        node
    }

    pub fn n_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = &GraphNode<T,U>> {
        self.nodes.iter()
    }

    pub fn edges_iter(&self) -> impl Iterator<Item = &GraphEdge<T,U>> {
        self.edges.iter().map(|(_, v)| v)
    }

    pub fn node_at(&self, i : usize) -> GraphNode<T,U> {
        Arc::clone(&self.nodes[i])
    }

    pub fn edge_at(&self, i : usize, j : usize) -> Option<GraphEdge<T,U>> {
        let edge = self.edges.get(&(i,j));
        edge.map(Arc::clone)
    }

    pub fn has_edge(&self, i : usize, j : usize) -> bool {
        self.edges.contains_key(&(i,j))
    }

    fn edge_index(&self, from : &GraphNode<T,U>, to : &GraphNode<T,U>) -> (usize, usize) {
        (from.index, to.index)
    }

    fn save_edge(&mut self, from : &GraphNode<T,U>, to : &GraphNode<T,U>, edge : GraphEdge<T,U>) -> Option<GraphEdge<T,U>> {
        let index = self.edge_index(&from, &to);
        if self.edges.contains_key(&index) {
            return None;
        }
        from.add_out_edge(&edge);
        to.add_in_edge(&edge);
        self.edges.insert(index, Arc::clone(&edge));
        Some(edge)
    }

    pub fn make_edge(&mut self, from : &T, to : &T, weight : U) -> Option<GraphEdge<T,U>>
    where
        T : PartialEq
    {
        let mut e = Edge::orphan(weight);
        let mut node_from : Option<Arc<DataNode<T,U>>> = None;
        let mut node_to : Option<Arc<DataNode<T,U>>> = None;
        for node in self.nodes.iter() {
            if node.element == *from {
                e.set_node_from(node);
                node_from = Some(Arc::clone(node));
            }
            if node.element == *to {
                e.set_node_to(node);
                node_to = Some(Arc::clone(node));
            }
        }
        if !e.is_connected() {
            return None;
        }
        let e = Arc::new(e);
        self.save_edge(&node_from.unwrap(), &node_to.unwrap(), e)
    }

    pub fn connect(&mut self, from : &GraphNode<T,U>, to : &GraphNode<T,U>, weight : U) -> Option<GraphEdge<T,U>> {
        let e = Edge::data_edge(from, to, weight);
        let e = Arc::new(e);
        from.add_out_edge(&e);
        to.add_in_edge(&e);
        self.save_edge(&from, &to, e)
    }

    pub fn are_connected(&mut self, from : &GraphNode<T,U>, to : &GraphNode<T,U>) -> bool {
        self.has_edge(from.index, to.index)
    }

    pub fn get_connection(&self, from : &GraphNode<T,U>, to : &GraphNode<T,U>) -> Option<GraphEdge<T,U>> {
        let edge = &self.edges.get(&self.edge_index(from, to));
        edge.map(Arc::clone)
    }

    pub fn disconnect(&mut self, from : &GraphNode<T,U>, to : &GraphNode<T,U>) {
        let index = self.edge_index(from, to);
        self.remove_edge_at(index);
    }

    pub fn remove_edge_at(&mut self, index : (usize, usize)) {
        if !self.edges.contains_key(&index) {
            return;
        }
        self.edges.remove(&index);

        let from = &self.nodes[index.0];
        let to = &self.nodes[index.1];

        let mut to_remove = None;
        for (i, edge) in from.out_edges.read().unwrap().iter().enumerate() {
            if !edge.has_target() {
                continue;
            }
            if edge.get_node_to().index == to.index {
                to_remove = Some(i);
                break;
            }
        }
        if let Some(i) = to_remove {
            from.out_edges.write().unwrap().remove(i);
        }
        let mut to_remove = None;
        for (i, edge) in to.in_edges.read().unwrap().iter().enumerate() {
            if !edge.has_source() {
                continue;
            }
            if edge.get_node_from().index == from.index {
                to_remove = Some(i);
                break;
            }
        }
        if let Some(i) = to_remove {
            to.in_edges.write().unwrap().remove(i);
        }
    }

    pub fn remove_node(&mut self, node : &GraphNode<T,U>) {
        let n = self.n_nodes();
        for j in 0..n {
            self.remove_edge_at((node.index, j));
            self.remove_edge_at((j, node.index));
        }
    }

    pub fn make_edge_when<F>(&mut self, filter : F, weight : U) -> Vec<GraphEdge<T,U>>
    where
        F : Fn (&T,&T) -> bool,
        U : Clone
    {
        let mut res = Vec::new();
        for from in self.nodes.iter() {
            for to in self.nodes.iter() {
                let index = self.edge_index(&from, &to);
                if self.edges.contains_key(&index) { continue };
                if !filter(&from.element, &to.element) { continue };
                let e = Edge::data_edge(from, to, weight.clone());
                let e = Arc::new(e);
                from.add_out_edge(&e);
                to.add_in_edge(&e);
                self.edges.insert(index, Arc::clone(&e));
                res.push(e);
            }
        }
        res
    }

    pub fn find<F>(&self, filter : F) -> Vec<GraphNode<T,U>>
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

    pub fn find_first<F>(&self, filter : F) -> Option<GraphNode<T,U>>
        where F : Fn(&T) -> bool
    {
        for node in self.nodes.iter() {
            if filter(&node.element) {
                return Some(Arc::clone(node));
            }
        }
        None
    }

    pub fn get_node(&self, value : &T) -> Option<GraphNode<T,U>>
        where T : PartialEq
    {
        self.find_first(|x| *x == *value)
    }

    // Implementation of the Floyd-Warshall algorithm ----------------------------------

    pub fn all_shortest_paths(&self) -> DMatrix<U>
    where
        U : Add<Output = U> + PartialOrd + Zero + Bounded + Scalar
    {
        self.floyd_warshall(U::clone, U::max_value())
    }

    pub fn all_shortest_float_paths<F>(&self, weight : F) -> DMatrix<f64>
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
        U : Add<Output = U> + PartialOrd + Zero + Bounded + Scalar
    {
        let distances = self.all_shortest_paths();
        let mut res = Digraph {
            nodes : self.nodes.clone(),
            edges : Default::default()
        };
        res.create_relations(distances);
        res
    }

    // ---------------------------------------------------------------------------------

    // Implementation of the Dijkstra algorithm ----------------------------------------

    pub fn shortest_path(&self, from : &GraphNode<T,U>, target : &GraphNode<T,U>)
        -> Option<(U, Vec<GraphEdge<T,U>>)>
    where
        U : Add<Output = U> + PartialOrd + Zero + Bounded + Scalar
    {
        self.shortest_weighted_path(from, target, U::clone, U::max_value())
    }

    pub fn shortest_paths_from(&self, from : &GraphNode<T,U>)
        -> (Vec<U>, Vec<Vec<GraphEdge<T,U>>>)
    where
        U : Add<Output = U> + PartialOrd + Zero + Bounded + Scalar
    {
        self.shortest_weighted_paths_from(from, U::clone, U::max_value())
    }

    pub fn shortest_weighted_path<F,V>(
        &self, from : &GraphNode<T,U>, target : &GraphNode<T,U>,
        weight : F, no_edge : V
    )
        -> Option<(V, Vec<GraphEdge<T,U>>)>
    where
        F : Fn(&U) -> V,
        V : Add<Output = V> + PartialOrd + Zero + Scalar
    {
        let (mut dists, mut traces) = self.dijkstra(from, Some(target), weight, no_edge.clone());
        if dists[target.index] < no_edge {
            return Some((dists.remove(target.index), traces.remove(target.index)));
        }
        None
    }

    pub fn shortest_weighted_paths_from<F,V>(
        &self, from : &GraphNode<T,U>, weight : F, no_edge : V
    )
        -> (Vec<V>, Vec<Vec<GraphEdge<T,U>>>)
    where
        F : Fn(&U) -> V,
        V : Add<Output = V> + PartialOrd + Zero + Scalar
    {
        self.dijkstra(from, None, weight, no_edge.clone())
    }

    pub fn dijkstra<F,V>(
        &self, from : &GraphNode<T,U>, target : Option<&GraphNode<T,U>>,
        weight : F, no_edge : V
    )
        -> (Vec<V>, Vec<Vec<GraphEdge<T,U>>>)
    where
        F : Fn(&U) -> V,
        V : Add<Output = V> + PartialOrd + Zero + Scalar
    {
        let i = from.index;
        let target = target.map(|t| t.index);
        let n = self.n_nodes();
        let mut dists = Vec::new();
        let mut traces = vec![Vec::new() ; n];
        let matrix = self.make_weight_matrix(weight, no_edge.clone());

        let mut added = vec![false ; n];
        added[i] = true;
        let mut min_w = no_edge.clone();
        let mut min_j = usize::MAX;
        for j in 0..n {
            if i == j { continue }
            let value = matrix[(i,j)].clone();
            if value < min_w {
                min_w = value.clone();
                min_j = j;
            }
            if let Some(e) = self.edge_at(i, j) {
                traces[j].push(e);
            }
            dists.push(value);
        }

        while min_j < usize::MAX && (target.is_none() || !added[target.unwrap()]){
            added[min_j] = true;
            let pre = min_j;
            min_w = no_edge.clone();
            min_j = usize::MAX;
            for j in 0..n {
                if added[j] { continue }
                let new_arc = dists[pre].clone() + matrix[(pre,j)].clone();
                if new_arc < dists[j] {
                    dists[j] = new_arc;
                    traces[j] = traces[pre].clone();
                    traces[j].push(self.edge_at(i, j).unwrap());
                }
                if dists[j] < min_w {
                    min_w = dists[j].clone();
                    min_j = j;
                }
            }
        }

        (dists, traces)
    }

    // ---------------------------------------------------------------------------------

    pub fn is_positive(&self) -> bool
    where
        U : Zero + PartialOrd + Clone
    {
        self.is_positively_weighted(U::clone)
    }

    pub fn is_positively_weighted<F,V>(&self, weight : F) -> bool
    where
        F : Fn(&U) -> V,
        V : PartialOrd + Zero
    {
        for (_, edge) in self.edges.iter() {
            if weight(edge.data()) < V::zero() {
                return false;
            }
        }
        true
    }

    pub fn from_matrix(elements : Vec<T>, relations : DMatrix<U>) -> Self
    where
        U : PartialOrd + Clone + Bounded + Zero
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
        V : Zero + Scalar
    {
        let n_nodes = self.nodes.len();
        DMatrix::from_fn(n_nodes, n_nodes, |i,j| {
            if i == j { V::zero() }
            else {
                if let Some(edge) = &self.edges.get(&(i,j)) {
                    weight(&edge.weight)
                }
                else {
                    no_edge.clone()
                }
            }
        })
    }

    pub fn has_loop(&self) -> bool {
        let mut seen = vec![false ; self.n_nodes()];
        loop {
            let next_index = seen.iter().position(|x| !*x);
            let Some(next_index) = next_index else {
                return false;
            };
            let traversal = GraphTraversal::dfs(self.nodes[next_index].clone());
            for node in traversal {
                if seen[node.index] {
                    return true;
                }
                seen[node.index] = true;
            }
        }
    }

    pub fn create_relations(&mut self, relations : DMatrix<U>)
    where
        U : PartialOrd + Clone + Bounded + Zero
    {
        let n_nodes = self.nodes.len();
        for (n, w) in relations.iter().enumerate() {
            let i = n / n_nodes;
            let j = n % n_nodes;
            let index = (i,j);
            if *w >= U::max_value() || (w.is_zero() && i == j) { // Max length = INF, min length to self = no edge
                continue;
            }
            if self.edges.contains_key(&index) {
                continue;
            }
            let from = &self.nodes[i];
            let to = &self.nodes[j];
            let e = Edge::data_edge(from, to, w.clone());
            let e = Arc::new(e);
            from.add_out_edge(&e);
            to.add_in_edge(&e);
            self.edges.insert(index, e);
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
        Digraph { nodes, edges : Default::default() }
    }
}

impl<T : ToString, U> Digraph<T,U> {

    pub fn labelize(&mut self)
        where T : ToString, U : Clone
    {
        for node in self.nodes.iter() {
            node.clear_edges();
        }
        let mut new_edges = BTreeMap::new();
        for (_, edge) in self.edges.iter() {
            let mut new_edge = Edge::orphan(edge.weight.clone());
            let source = edge.get_node_from();
            new_edge.set_node_from(&source);
            new_edge.from = Some(source.get_label());
            let target = edge.get_node_to();
            new_edge.set_node_to(&target);
            new_edge.to = Some(target.get_label());
            let new_edge = Arc::new(new_edge);
            source.add_out_edge(&new_edge);
            target.add_in_edge(&new_edge);
            let index = self.edge_index(&source, &target);
            new_edges.insert(index, new_edge);
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
                graph.make_edge(&i, &j, matrix[(i,j)]);
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
