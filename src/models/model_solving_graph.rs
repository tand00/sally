use crate::{models::*, solution::Solution, verification::query::Query};

use self::translation::Translation;

pub struct ModelSolvingGraph {
    pub models : Vec<SimpleNode<ModelMeta>>,
    pub translations : Vec<Box<dyn Translation>>,
    pub solutions : Vec<Box<dyn Solution>>,
    pub edges : Vec<Edge<usize>>,
}

impl ModelSolvingGraph {
    
    pub fn new() -> Self {
        ModelSolvingGraph {
            models : Vec::new(),
            translations : Vec::new(),
            solutions : Vec::new(),
            edges : Vec::new()
        }
    }

    pub fn register_model(&mut self, meta : ModelMeta) {
        let node = SimpleNode::from(meta);
        self.models.push(node);
    }

    pub fn register_translation(&mut self, translation : Box<dyn Translation>) {
        self.translations.push(translation)
    }

    pub fn register_solution(&mut self, solution : Box<dyn Solution>) {
        self.solutions.push(solution)
    }

    pub fn solve(&mut self, model : &dyn Any, query : &Query) {
        
    }

    pub fn compile(&mut self) {
        
    }

}