use std::{cmp::min, ops::Add};

use crate::computation::DBM;

use super::{node::SimpleNode, time::TimeBound, Edge, Label, Model, Node};

// T is the type to be stored in Nodes, while U is the type of edges weights
pub struct Digraph<T,U> {
    pub edges : Vec<Edge<U>>,
    pub nodes : Vec<SimpleNode<T>>,
    matrix : Vec<Vec<Option<usize>>>
}

impl<T,U> Digraph<T,U> {

    pub fn new() -> Self {
        Digraph {
            edges : Vec::new(),
            nodes : Vec::new(),
            matrix : Vec::new()
        }
    }

    pub fn make_node(&mut self, value : T) {
        let node = SimpleNode::from(value);
        self.nodes.push(node);
        let new_vec : Vec<Option<usize>> = vec![None ; self.nodes.len()];
        self.matrix.push(new_vec);
    }

}

impl<T : Clone, U> Digraph<T,U> {

    pub fn from(data : Vec<T>) -> Self {
        let nodes : Vec<SimpleNode<T>> = data.iter().map(|x| SimpleNode::from(x.clone())).collect();
        let matrix : Vec<Option<usize>> = vec![None ; nodes.len()];
        let matrix : Vec<Vec<Option<usize>>> = vec![matrix ; nodes.len()];
        Digraph {
            nodes,
            edges : Vec::new(),
            matrix
        }
    }

}

impl<T : ToString + PartialEq, U> Digraph<T,U> {

    pub fn make_edge(&mut self, from : T, to : T, weight : U) {
        let mut e = Edge::new_weighted(
            Label::new(), 
            Label::new(), 
            weight);
        let mut input = None;
        let mut output = None;
        for (i, n) in self.nodes.iter_mut().enumerate() {
            if n.element == from {
                e.set_node_from(i);
                e.from = Some(n.get_label());
                n.out_edges.push(self.edges.len());
                input = Some(i);
            }
            if n.element == to {
                e.set_node_to(i);
                e.to = Some(n.get_label());
                n.in_edges.push(self.edges.len());
                output = Some(i);
            }
            if let (Some(a), Some(b)) = (input, output) {
                self.matrix[a][b] = Some(self.edges.len());
                break;
            }
        }
        self.edges.push(e);
    }

}

impl<T, U : Add<Output = U> + Ord + Clone> Digraph<T,U> {

    // Implementation of the Floyd-Warshall algorithm
    pub fn shortest_paths(&self, min_length : U, max_length : U) -> Vec<Vec<U>> {
        let n_nodes = self.nodes.len();
        let distances = vec![max_length.clone() ; n_nodes];
        let mut distances = vec![distances ; n_nodes];
        for i in 0..n_nodes {
            distances[i][i] = min_length.clone();
            let node = &self.nodes[i];
            for edge_i in node.out_edges.iter() {
                let j = self.edges[*edge_i].node_to();
                let weight = self.edges[*edge_i].weight.clone();
                distances[i][j] = weight;
            }
        }
        for k in 0..n_nodes {
            for i in 0..n_nodes {
                for j in 0..n_nodes {
                    // Potential optimization, but restrict to null min length 
                    /*if (i == j) || (i == k) || (j == k) {
                        continue;
                    }*/
                    distances[i][j] = min(
                        distances[i][j].clone(),
                        distances[i][k].clone() + distances[k][j].clone()
                    );
                }
            }
        }
        distances
    }

}

impl<T : ToString + Clone, U : Ord + Clone> Digraph<T,U> {

    pub fn from_matrix(elements : Vec<T>, relations : Vec<Vec<U>>, min_length : U, max_length : U) -> Self {
        let mut graph = Self::from(elements);
        graph.create_relations(relations, min_length, max_length);
        graph
    }

    pub fn create_relations(&mut self, relations : Vec<Vec<U>>, min_length : U, max_length : U) {
        if relations.len() > self.nodes.len() || relations[0].len() > self.nodes.len() {
            panic!("Can't create relations to out of bound nodes !");
        }
        for (i,v) in relations.iter().enumerate() {
            for (j,w) in v.iter().enumerate() {
                let from = &self.nodes[i];
                let to = &self.nodes[j];
                let mut e = Edge::new_weighted(
                    from.get_label(), 
                    to.get_label(), 
                    w.clone());
                if *w >= max_length || (*w <= min_length && i == j) { // Max length = INF, min length to self = no edge
                    continue;
                }
                e.set_node_from(i);
                e.set_node_to(j);
                self.matrix[i][j] = Some(self.edges.len());
                self.edges.push(e);
            }
        }
    }

}

impl<T : ToString + Clone, U : Add<Output = U> + Ord + Clone> Digraph<T,U> {
    pub fn shortest_digraph(&self, min_length : U, max_length : U) -> Self {
        let distances = self.shortest_paths(min_length.clone(), max_length.clone());
        let matrix : Vec<Option<usize>> = vec![None ; self.nodes.len()];
        let matrix : Vec<Vec<Option<usize>>> = vec![matrix ; self.nodes.len()];
        let mut res = Digraph {
            nodes : self.nodes.clone(),
            edges : Vec::new(),
            matrix
        };
        res.create_relations(distances, min_length, max_length);
        res
    }
}

impl<T,U> Model for Digraph<T,U> {
    type State = usize;
    type Action = usize;

    fn next(&self, state : Self::State, action : Self::Action) -> (Self::State, Vec<usize>) {
        let node = &self.nodes[state];
        let edge = &self.edges[action];
        if !node.out_edges.contains(&action) {
            panic!("Can't fire action Digraph action from node");
        }
        let new_state = edge.node_to();
        let actions = self.nodes[new_state].out_edges.clone();
        (new_state, actions)
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
        DBM::new(self.nodes.len())
    }

}