use std::{cell::RefCell, rc::{Rc, Weak}};

use serde::{Deserialize, Serialize};

use super::{ComponentPtr, Edge, Label};

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
    pub out_edges : Vec<Rc<Edge<U, Self, Self>>>, // I must admit that this is not very pretty
    pub in_edges : Vec<Rc<Edge<U, Self, Self>>>,
    pub index : usize,
}

impl<T, U> DataNode<T, U> {

    pub fn from(element : T) -> Self {
        DataNode {
            element,
            out_edges : Vec::new(),
            in_edges : Vec::new(),
            index : 0
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
