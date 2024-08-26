use std::{collections::HashMap, fmt::Display, sync::Arc};

use digraph::{search_strategy::{BreadthFirst, GraphTraversal, UniqFilteredNeighbors}, Digraph};

use crate::{io::{ModelIOError, ModelLoader, ModelLoadingResult, ModelWriter, ModelWritingResult}, log, models::*, solution::{self, Solution, SolverResult}, translation::Translation, verification::query::Query};

use self::node::DataNode;

pub enum SolverGraphNode {
    AnySemantics,
    Semantics(ModelMeta),
    Solution(Box<dyn Solution>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SolverGraphEdge {
    Translation,
    Feature
}

pub struct TranslationsSet(pub Vec<Box<dyn Translation>>);

impl Display for SolverGraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub type ModelSolvingGraphNode = Arc<DataNode<SolverGraphNode, SolverGraphEdge>>;
pub type ModelSolvingGraphEdge = Arc<Edge<SolverGraphEdge, DataNode<SolverGraphNode, SolverGraphEdge>, DataNode<SolverGraphNode, SolverGraphEdge>>>;

pub struct ModelSolvingGraph {
    pub node_any : ModelSolvingGraphNode,
    pub semantics : HashMap<Label, ModelSolvingGraphNode>,
    pub solutions : Vec<ModelSolvingGraphNode>,
    pub writers : HashMap<Label, Box<dyn ModelWriter>>,
    pub loaders : HashMap<Label, Box<dyn ModelLoader>>,
    pub translations : Vec<ModelSolvingGraphEdge>,
    pub graph : Digraph<SolverGraphNode, SolverGraphEdge>
}

impl ModelSolvingGraph {
    
    pub fn new() -> Self {
        let mut graph = Digraph::new();
        let node_any = graph.make_node(SolverGraphNode::AnySemantics);
        ModelSolvingGraph {
            graph, node_any,
            semantics : HashMap::new(),
            solutions : Vec::new(),
            writers : HashMap::new(),
            loaders : HashMap::new(),
            translations : Vec::new(),
        }
    }

    pub fn register_model(&mut self, meta : ModelMeta) {
        let label = meta.name.clone();
        let node = self.graph.make_node(SolverGraphNode::Semantics(meta));
        self.semantics.insert(label, node);
    }

    pub fn register_translation(&mut self, translation : impl Translation + 'static) {
        let meta = translation.get_meta();
        let node_in = self.semantics.get(&meta.input).unwrap_or(&self.node_any);
        let node_out = self.semantics.get(&meta.output).unwrap_or(&self.node_any);

    }

    pub fn register_solution(&mut self, solution : impl Solution + 'static) {
        let solution = Box::new(solution);
        self.solutions.push(self.graph.make_node(SolverGraphNode::Solution(solution)));
    }

    pub fn register_loader(&mut self, loader : impl ModelLoader + 'static) {
        let ext = loader.get_meta().ext;
        self.loaders.insert(ext, Box::new(loader));
    }

    pub fn register_writer(&mut self, writer : impl ModelWriter + 'static) {
        let ext = writer.get_meta().ext;
        self.writers.insert(ext, Box::new(writer));
    }

    pub fn load_file(&self, path : String) -> ModelLoadingResult {
        let ext : Vec<&str> = path.split('.').collect();
        let Some(ext) = ext.last() else { return Err(ModelIOError) };
        let ext = Label::from(ext.to_owned());
        log::pending(format!("Loading file [{path}]"));
        log::continue_info(format!("File extension is [{ext}]"));
        let Some(loader) = self.loaders.get(&ext) else {
            log::error("No loader registered for this extension !");
            return Err(ModelIOError)
        };
        log::continue_info(format!("Using loader [{}]", loader.get_meta().name));
        let res = loader.load_file(path)?;
        log::positive("Loaded file !");
        Ok(res)
    }

    pub fn write_file(&self, path : String, model : &dyn ModelObject, initial : Option<InitialMarking>) -> ModelWritingResult {
        let ext : Vec<&str> = path.split('.').collect();
        let Some(ext) = ext.last() else { return Err(ModelIOError) };
        let ext = Label::from(ext.to_owned());
        log::pending(format!("Writing model to file [{path}]"));
        log::continue_info(format!("File extension is [{ext}]"));
        let Some(writer) = self.writers.get(&ext) else {
            log::error("No writer registered for this extension !");
            return Err(ModelIOError)
        };
        let writer_meta = writer.get_meta();
        log::continue_info(format!("Using writer [{}]", writer_meta.name));
        if writer_meta.input != lbl("any") && writer_meta.input != model.get_model_meta().name {
            log::error("The writer seems to be incompatible with the model type !");
            return Err(ModelIOError);
        }
        let res = writer.write_file(path, model, initial)?;
        log::positive("Written model to file !");
        Ok(res)
    }

    pub fn find_semantics(&self, model : &dyn ModelObject) -> Option<ModelSolvingGraphNode> {
        self.semantics.get(&model.get_model_meta().name).map(Arc::clone)
    }

    pub fn solve(&mut self, model : &dyn ModelObject, query : &Query) -> SolverResult {  
        let Some(node) = self.find_semantics(model) else {
            return SolverResult::SolverError;
        };
        let filter = UniqFilteredNeighbors::new(|e : &ModelSolvingGraphEdge| {
            e.weight == SolverGraphEdge::Translation
        });
        let traversal = GraphTraversal::new(
            node, BreadthFirst::new(), filter
        );
        for next_node in traversal {
            
        }
        SolverResult::BoolResult(true)
    }

}