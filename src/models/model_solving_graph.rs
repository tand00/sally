use std::fmt::Display;

use digraph::Digraph;

use crate::{io::{ModelLoader, ModelWriter}, models::*, solution::Solution, translation::Translation, verification::query::Query};

use self::node::DataNode;

pub enum SolverGraphNode {
    Semantics(ModelMeta),
    Solution(Box<dyn Solution>),
    Loader(Box<dyn ModelLoader>),
    Writer(Box<dyn ModelWriter>)
}

pub enum SolverGraphEdge {
    Translation(Box<dyn Translation>),
    Feature
}

impl Display for SolverGraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub struct ModelSolvingGraph {
    pub graph : Digraph<SolverGraphNode, SolverGraphEdge>
}

impl ModelSolvingGraph {
    
    pub fn new() -> Self {
        ModelSolvingGraph {
            graph : Digraph::new()
        }
    }

    pub fn register_model(&mut self, meta : ModelMeta) {
        self.graph.make_node(SolverGraphNode::Semantics(meta))
    }

    pub fn register_translation(&mut self, translation : Box<dyn Translation>) {
        
    }

    pub fn register_solution(&mut self, solution : Box<dyn Solution>) {
        self.graph.make_node(SolverGraphNode::Solution(solution))
    }

    pub fn register_loader(&mut self, loader : Box<dyn ModelLoader>) {
        self.graph.make_node(SolverGraphNode::Loader(loader))
    }

    pub fn register_writer(&mut self, writer : Box<dyn ModelWriter>) {
        self.graph.make_node(SolverGraphNode::Writer(writer))
    }

    pub fn solve(&mut self, model : &dyn Any, query : &Query) {
        
    }

    pub fn compile(&mut self) {
        
    }

}