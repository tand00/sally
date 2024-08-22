use std::{collections::VecDeque, marker::PhantomData};

use rand::{thread_rng, Rng};

use super::GraphNode;

pub trait SearchStrategy<T> {

    fn feed(&mut self, x : T);

    fn next(&mut self) -> Option<T>;

}

pub struct BreadthFirst<T> {
    vec : VecDeque<T>
}
impl<T> BreadthFirst<T> {
    pub fn new() -> Self {
        Self { vec : VecDeque::new() }
    }
}

pub struct DepthFirst<T> {
    vec : Vec<T>
}
impl<T> DepthFirst<T> {
    pub fn new() -> Self {
        Self { vec : Vec::new() }
    }
}

pub struct RandomSearch<T> {
    vec : Vec<T>
}
impl<T> RandomSearch<T> {
    pub fn new() -> Self {
        Self { vec : Vec::new() }
    }
}

impl<T> SearchStrategy<T> for BreadthFirst<T> {
    fn feed(&mut self, x : T) {
        self.vec.push_back(x)
    }
    fn next(&mut self) -> Option<T> {
        self.vec.pop_front()
    }
}

impl<T> SearchStrategy<T> for DepthFirst<T> {
    fn feed(&mut self, x : T) {
        self.vec.push(x)
    }
    fn next(&mut self) -> Option<T> {
        self.vec.pop()
    }
}

impl<T> SearchStrategy<T> for RandomSearch<T> {
    fn feed(&mut self, x : T) {
        self.vec.push(x)
    }

    fn next(&mut self) -> Option<T> {
        if self.vec.is_empty() {
            return None;
        }
        let mut rng = thread_rng();
        let index = rng.gen_range(0..self.vec.len());
        Some(self.vec.remove(index))
    }
}

pub trait NeighborsFinder<U> {
    fn neighbors(&mut self, x : &U) -> Vec<U>;
}

pub struct GraphTraversal<U, S : SearchStrategy<U>, N : NeighborsFinder<U>> {
    pub strategy : S,
    pub neighbors_finder : N,
    phantom : PhantomData<U>
}

impl<U, S : SearchStrategy<U>, N : NeighborsFinder<U>> GraphTraversal<U, S, N> {

    pub fn new(initial : U, mut strategy : S, neighbors_finder : N) -> Self {
        strategy.feed(initial);
        GraphTraversal {
            strategy, neighbors_finder, phantom : PhantomData
        }
    }

}

impl<U, S : SearchStrategy<U>, N : NeighborsFinder<U>> Iterator for GraphTraversal<U, S, N> {

    type Item = U;
    
    fn next(&mut self) -> Option<Self::Item> {
        let x = self.strategy.next();
        if let Some(x) = x {
            let neighbors = self.neighbors_finder.neighbors(&x);
            for neighbor in neighbors {
                self.strategy.feed(neighbor);
            }
            return Some(x);
        }
        None
    }

}

pub struct DigraphNeighbors;
impl<T,U> NeighborsFinder<GraphNode<T,U>> for DigraphNeighbors {
    fn neighbors(&mut self, x : &GraphNode<T,U>) -> Vec<GraphNode<T,U>> {
        x.out_edges.read().unwrap().iter().map(|e| {
            e.get_node_to()
        }).collect()
    }
}

pub struct UniqDigraphNeighbors {
    pub seen : Vec<bool>
}
impl UniqDigraphNeighbors {
    pub fn new() -> Self {
        Self { seen : Vec::new() }
    }
}
impl<T,U> NeighborsFinder<GraphNode<T,U>> for UniqDigraphNeighbors {
    fn neighbors(&mut self, x : &GraphNode<T,U>) -> Vec<GraphNode<T,U>> {
        x.out_edges.read().unwrap().iter().filter_map(|e| {
            let node = e.get_node_to();
            if self.seen[node.index] {
                None
            } else {
                self.seen[node.index] = true;
                Some(node)
            }
        }).collect()
    }
}

impl<T,U> GraphTraversal<GraphNode<T,U>, BreadthFirst<GraphNode<T,U>>, DigraphNeighbors> {
    pub fn bfs(initial : GraphNode<T,U>) -> Self {
        Self::new(initial, BreadthFirst::new(), DigraphNeighbors)
    }
}
impl<T,U> GraphTraversal<GraphNode<T,U>, DepthFirst<GraphNode<T,U>>, DigraphNeighbors> {
    pub fn dfs(initial : GraphNode<T,U>) -> Self {
        Self::new(initial, DepthFirst::new(), DigraphNeighbors)
    }
}
impl<T,U> GraphTraversal<GraphNode<T,U>, RandomSearch<GraphNode<T,U>>, DigraphNeighbors> {
    pub fn random(initial : GraphNode<T,U>) -> Self {
        Self::new(initial, RandomSearch::new(), DigraphNeighbors)
    }
}