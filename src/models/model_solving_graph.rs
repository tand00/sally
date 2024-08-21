use std::{fmt::Display, sync::{Arc, Mutex}};

use digraph::Digraph;

use crate::{io::{ModelLoader, ModelWriter}, models::*, solution::{Solution, SolverResult}, translation::Translation, verification::query::Query};

use self::node::DataNode;

pub enum SolverGraphNode {
    AnySemantics,
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

pub type ModelSolvingGraphNode = Arc<DataNode<SolverGraphNode, SolverGraphEdge>>;
pub type ModelSolvingGraphEdge = Arc<Edge<ModelSolvingGraphNode, SolverGraphEdge, ModelSolvingGraphNode>>;

pub struct ModelSolvingGraph {
    pub node_any : ModelSolvingGraphNode,
    pub semantics : Vec<ModelSolvingGraphNode>,
    pub solutions : Vec<ModelSolvingGraphNode>,
    pub writers : Vec<ModelSolvingGraphNode>,
    pub loaders : Vec<ModelSolvingGraphNode>,
    pub translations : Vec<ModelSolvingGraphEdge>,
    pub graph : Digraph<SolverGraphNode, SolverGraphEdge>
}

impl ModelSolvingGraph {
    
    pub fn new() -> Self {
        let graph = Digraph::from(vec![SolverGraphNode::AnySemantics]);
        let node_any = Arc::clone(&graph.nodes[0]);
        ModelSolvingGraph {
            graph, node_any,
            semantics : Vec::new(),
            solutions : Vec::new(),
            writers : Vec::new(),
            loaders : Vec::new(),
            translations : Vec::new(),
        }
    }

    pub fn register_model(&mut self, meta : ModelMeta) {
        self.semantics.push(self.graph.make_node(SolverGraphNode::Semantics(meta)));
    }

    pub fn register_translation(&mut self, translation : Box<dyn Translation>) {
        
    }

    pub fn register_solution(&mut self, solution : Box<dyn Solution>) {
        self.solutions.push(self.graph.make_node(SolverGraphNode::Solution(solution)));
    }

    pub fn register_loader(&mut self, loader : Box<dyn ModelLoader>) {
        self.loaders.push(self.graph.make_node(SolverGraphNode::Loader(loader)));
    }

    pub fn register_writer(&mut self, writer : Box<dyn ModelWriter>) {
        self.writers.push(self.graph.make_node(SolverGraphNode::Writer(writer)));
    }

    pub fn find_semantics(&self, model : &dyn ModelObject) -> Option<ModelSolvingGraphNode> {
        for node in self.semantics.iter() {
            let SolverGraphNode::Semantics(s) = &node.element else {
                return None;
            };
            if s.name == model.get_model_meta().name {
                return Some(Arc::clone(node));
            }
        }
        None
    }

    pub fn solve(&mut self, model : &dyn ModelObject, query : &Query) -> SolverResult {  
        let Some(node) = self.find_semantics(model) else {
            return SolverResult::SolverError;
        };

        let mut available_solutions = Vec::new();
        let mut available_translations = Vec::new();

        for edge in node.out_edges.read().unwrap().iter() {
            let edge_data = edge.data();
            if let SolverGraphEdge::Translation(t) = edge_data {
                available_translations.push(t);
                continue;
            };
            if !edge.has_target() {
                continue;
            }
            let target = edge.get_node_to();
            if let SolverGraphNode::Solution(_) = &target.element {
                available_solutions.push(target);
            }
        }

        SolverResult::BoolResult(true)
    }

}