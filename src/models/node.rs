use std::sync::{Arc, RwLock};

use super::{Edge, Label};

/// Generic trait that should be implemented by all types of nodes (useless at the moment)
pub trait Node {
    fn get_label(&self) -> Label;
}

impl Node for usize {
    fn get_label(&self) -> Label {
        Label::from(self.to_string())
    }
}

// T is the data type carried by the node, U is the data type carried by the edges
pub struct DataNode<T, U> {
    pub element : T,
    pub out_edges : RwLock<Vec<Arc<Edge<U, Self, Self>>>>, 
    pub in_edges : RwLock<Vec<Arc<Edge<U, Self, Self>>>>,
    pub index : usize,
}

impl<T, U> DataNode<T, U> {

    pub fn from(element : T) -> Self {
        DataNode {
            element,
            out_edges : Default::default(),
            in_edges : Default::default(),
            index : 0,
        }
    }

}

impl<T : ToString + 'static, U> Node for DataNode<T, U> {

    fn get_label(&self) -> Label {
        Label::from(self.element.to_string())
    }

}

impl<T : Clone, U> Clone for DataNode<T, U> {
    fn clone(&self) -> Self {
        DataNode::from(self.element.clone())
    }
}

impl<T : PartialEq, U> PartialEq for DataNode<T, U> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}
