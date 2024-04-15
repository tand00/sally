use crate::models::*;

pub struct ModelSolvingGraph {
    pub models : Vec<SimpleNode<ModelMeta>>,
    pub translations : Vec<Edge<u16>>,
}

impl ModelSolvingGraph {
    
    pub fn new() -> Self {
        ModelSolvingGraph {
            models : Vec::new(),
            translations : Vec::new()
        }
    }

    pub fn register_model(&mut self, meta : ModelMeta) {
        let node = SimpleNode::from(meta);
        self.models.push(node);
    }

    pub fn compute_translations(&mut self) {

    }

}